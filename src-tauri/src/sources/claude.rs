//! 解析 Claude Code 本地会话（~/.claude/projects/<dir>/<uuid>.jsonl）

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::models::{
    now_ms, status_for_age, truncate_chars, SessionDetail, SessionSummary, TodoItem,
};

pub fn projects_dir() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".claude").join("projects"))
}

/// 列出最近活跃的 Claude 会话（按 mtime 过滤，再解析文件内容）
pub fn scan_sessions() -> Vec<SessionDetail> {
    let Some(root) = projects_dir() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let Ok(project_dirs) = fs::read_dir(&root) else {
        return Vec::new();
    };
    for dir in project_dirs.flatten() {
        let Ok(files) = fs::read_dir(dir.path()) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Ok(meta) = file.metadata() else { continue };
            let Ok(modified) = meta.modified() else { continue };
            let age = modified.elapsed().map(|d| d.as_secs()).unwrap_or(u64::MAX);
            let Some(status) = status_for_age(age) else {
                continue;
            };
            if let Some(detail) = parse_session(&path, status, age) {
                // 跳过隐藏目录中的会话（如 claude-mem 等插件的后台 agent），它们不是用户自己的任务
                if detail.summary.project_path.contains("/.") {
                    continue;
                }
                out.push(detail);
            }
        }
    }
    out
}

struct TaskListBuilder {
    tasks: Vec<TodoItem>,
}

impl TaskListBuilder {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// TaskCreate 的 taskId 按创建顺序从 "1" 递增分配
    fn on_create(&mut self, input: &Value) {
        let subject = input
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or("(未命名任务)");
        self.tasks.push(TodoItem {
            content: subject.to_string(),
            status: "pending".into(),
        });
    }

    fn on_update(&mut self, input: &Value) {
        let Some(idx) = input
            .get("taskId")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<usize>().ok())
            .and_then(|n| n.checked_sub(1))
        else {
            return;
        };
        if let Some(task) = self.tasks.get_mut(idx) {
            if let Some(status) = input.get("status").and_then(Value::as_str) {
                task.status = status.to_string();
            }
            if let Some(subject) = input.get("subject").and_then(Value::as_str) {
                task.content = subject.to_string();
            }
        }
    }

    /// TodoWrite 每次都是全量快照，直接替换
    fn on_todo_write(&mut self, input: &Value) {
        let Some(todos) = input.get("todos").and_then(Value::as_array) else {
            return;
        };
        self.tasks = todos
            .iter()
            .map(|t| TodoItem {
                content: t
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                status: t
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("pending")
                    .to_string(),
            })
            .collect();
    }
}

fn parse_session(path: &Path, status: &str, age_secs: u64) -> Option<SessionDetail> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut title: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut git_branch: Option<String> = None;
    let mut started_at: Option<String> = None;
    let mut message_count: u32 = 0;
    let mut latest_message = String::new();
    let mut current_activity = String::new();
    let mut outputs: Vec<String> = Vec::new();
    let mut tasks = TaskListBuilder::new();
    let mut first_user_text: Option<String> = None;
    // 已发出但还没有 tool_result 的交互工具调用（id → 问题摘要），
    // 解析完仍有剩余说明会话正停在确认提示上
    let mut pending_interactive: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    // 已发出但还没有 tool_result 的权限类工具调用（id → 描述）。
    // 权限确认在 jsonl 里只是普通 tool_use，无等待标记，只能结合文件静默时长推断：
    // 编辑类工具执行近乎瞬时，静默超 30s 基本就是停在权限提示上；
    // Bash 可能本身长时间运行，放宽到 90s（长命令会误报一次"可能在等待"，可接受）
    let mut pending_permission: Vec<(String, String, bool)> = Vec::new();

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue; // 跳过无法解析的行，不中断整个会话
        };
        // 子代理（sidechain）的消息不计入主会话视图
        if v.get("isSidechain").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        // CLI 注入的元消息（如 resume 时的 "Continue from where you left off."）
        if v.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        // 合成回复（如 "No response requested."），非模型真实输出
        if v
            .get("message")
            .and_then(|m| m.get("model"))
            .and_then(Value::as_str)
            == Some("<synthetic>")
        {
            continue;
        }
        match v.get("type").and_then(Value::as_str) {
            Some("ai-title") => {
                if let Some(t) = v.get("aiTitle").and_then(Value::as_str) {
                    title = Some(t.to_string());
                }
            }
            Some("user") => {
                if v.get("message").is_none() {
                    continue;
                }
                message_count += 1;
                record_common_fields(&v, &mut cwd, &mut git_branch, &mut started_at);
                let content = v.get("message").and_then(|m| m.get("content"));
                if first_user_text.is_none() {
                    if let Some(text) = content.and_then(Value::as_str) {
                        if !text.starts_with('<') {
                            first_user_text = Some(truncate_chars(text, 40));
                        }
                    }
                }
                // tool_result 出现表示对应的工具调用已被回应
                if let Some(blocks) = content.and_then(Value::as_array) {
                    for block in blocks {
                        if block.get("type").and_then(Value::as_str) == Some("tool_result") {
                            if let Some(id) = block.get("tool_use_id").and_then(Value::as_str) {
                                pending_interactive.remove(id);
                                pending_permission.retain(|(pid, _, _)| pid != id);
                            }
                        }
                    }
                }
            }
            Some("assistant") => {
                message_count += 1;
                record_common_fields(&v, &mut cwd, &mut git_branch, &mut started_at);
                let Some(content) = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                for block in content {
                    match block.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            if let Some(text) = block.get("text").and_then(Value::as_str) {
                                if !text.trim().is_empty() {
                                    latest_message = truncate_chars(text, 200);
                                    current_activity = "正在回复".into();
                                }
                            }
                        }
                        Some("tool_use") => {
                            let name = block
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or("(工具)");
                            let input = block.get("input").unwrap_or(&Value::Null);
                            match name {
                                "AskUserQuestion" => {
                                    if let Some(id) = block.get("id").and_then(Value::as_str) {
                                        let q = input
                                            .get("questions")
                                            .and_then(Value::as_array)
                                            .and_then(|qs| qs.first())
                                            .and_then(|q| q.get("question"))
                                            .and_then(Value::as_str)
                                            .unwrap_or("等待选择");
                                        pending_interactive
                                            .insert(id.to_string(), truncate_chars(q, 80));
                                    }
                                }
                                "ExitPlanMode" => {
                                    if let Some(id) = block.get("id").and_then(Value::as_str) {
                                        pending_interactive
                                            .insert(id.to_string(), "等待批准实施计划".into());
                                    }
                                }
                                "TaskCreate" => tasks.on_create(input),
                                "TaskUpdate" => tasks.on_update(input),
                                "TodoWrite" => tasks.on_todo_write(input),
                                "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => {
                                    if let Some(fp) = input
                                        .get("file_path")
                                        .or_else(|| input.get("notebook_path"))
                                        .and_then(Value::as_str)
                                    {
                                        if !outputs.iter().any(|o| o == fp) {
                                            outputs.push(fp.to_string());
                                        }
                                    }
                                    if let Some(id) = block.get("id").and_then(Value::as_str) {
                                        pending_permission.push((
                                            id.to_string(),
                                            describe_tool(name, input),
                                            false,
                                        ));
                                    }
                                }
                                "Bash" => {
                                    if let Some(id) = block.get("id").and_then(Value::as_str) {
                                        pending_permission.push((
                                            id.to_string(),
                                            describe_tool(name, input),
                                            true,
                                        ));
                                    }
                                }
                                _ => {}
                            }
                            current_activity = describe_tool(name, input);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let project_path = cwd.unwrap_or_default();
    let title = title
        .or(first_user_text)
        .unwrap_or_else(|| project_name(&project_path));

    // 还有未回应的交互工具调用 → 会话停在确认提示上；
    // 否则若有未回应的权限类调用且文件静默超阈值 → 大概率停在权限提示上
    const EDIT_PERMISSION_WAIT_SECS: u64 = 30;
    const BASH_PERMISSION_WAIT_SECS: u64 = 90;
    let pending_question = pending_interactive.into_values().next().or_else(|| {
        pending_permission.last().and_then(|(_, desc, is_bash)| {
            let threshold = if *is_bash {
                BASH_PERMISSION_WAIT_SECS
            } else {
                EDIT_PERMISSION_WAIT_SECS
            };
            (age_secs >= threshold).then(|| format!("可能在等待权限确认 — {desc}"))
        })
    });
    let status = if pending_question.is_some() {
        "waiting"
    } else {
        status
    };

    let summary = SessionSummary {
        file_path: path.to_string_lossy().into_owned(),
        provider: "claude".into(),
        title,
        project_path,
        status: status.to_string(),
        started_at,
        last_activity_at: now_ms() - (age_secs as i64) * 1000,
        message_count,
        current_activity,
        pending_question,
    };
    Some(SessionDetail {
        summary,
        todos: tasks.tasks,
        outputs,
        latest_message,
        git_branch,
    })
}

fn record_common_fields(
    v: &Value,
    cwd: &mut Option<String>,
    git_branch: &mut Option<String>,
    started_at: &mut Option<String>,
) {
    if let Some(c) = v.get("cwd").and_then(Value::as_str) {
        *cwd = Some(c.to_string());
    }
    if let Some(b) = v.get("gitBranch").and_then(Value::as_str) {
        if !b.is_empty() {
            *git_branch = Some(b.to_string());
        }
    }
    if started_at.is_none() {
        if let Some(ts) = v.get("timestamp").and_then(Value::as_str) {
            *started_at = Some(ts.to_string());
        }
    }
}

pub(crate) fn describe_tool(name: &str, input: &Value) -> String {
    let target = match name {
        "Write" | "Edit" | "MultiEdit" | "Read" => input
            .get("file_path")
            .and_then(Value::as_str)
            .and_then(|p| Path::new(p).file_name())
            .map(|f| f.to_string_lossy().into_owned()),
        "Bash" => input
            .get("description")
            .or_else(|| input.get("command"))
            .and_then(Value::as_str)
            .map(|s| truncate_chars(s, 30)),
        _ => None,
    };
    match target {
        Some(t) => format!("{name}: {t}"),
        None => name.to_string(),
    }
}

pub fn project_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}
