//! 会话管理页：全部历史会话列表 + 单会话对话内容提取
//!
//! 列表阶段只读每个文件头部若干行取标题/项目目录（性能约束），
//! 对话内容在用户选中某个会话后按需全文解析。

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::models::{now_ms, truncate_chars, RUNNING_SECS};

/// 列表阶段每个文件最多读取的行数
const HEAD_LINES: usize = 100;
/// 单条消息文本上限，避免超长 payload
const MSG_MAX_CHARS: usize = 2000;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistorySession {
    pub file_path: String,
    pub provider: String,
    pub title: String,
    pub project_path: String,
    /// epoch 毫秒
    pub last_activity_at: i64,
    /// mtime 距今 < RUNNING_SECS，视为进行中（恢复按钮置灰）
    pub is_active: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// "user" | "assistant" | "tool"
    pub role: String,
    pub text: String,
    /// ISO 时间戳，缺省为 None
    pub at: Option<String>,
}

/// 两个数据源全部 jsonl，按 mtime 倒序
fn all_session_files() -> Vec<(PathBuf, u64, &'static str)> {
    let mut out: Vec<(PathBuf, u64, &'static str)> = Vec::new();

    if let Some(root) = super::claude::projects_dir() {
        if let Ok(dirs) = fs::read_dir(&root) {
            for dir in dirs.flatten() {
                let Ok(files) = fs::read_dir(dir.path()) else {
                    continue;
                };
                for file in files.flatten() {
                    let path = file.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                        continue;
                    }
                    let Some(age) = file_age(&path) else { continue };
                    out.push((path, age, "claude"));
                }
            }
        }
    }

    for (path, age) in super::codex::recent_files(u64::MAX) {
        out.push((path, age, "codex"));
    }

    out.sort_by_key(|(_, age, _)| *age);
    out
}

fn file_age(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(modified.elapsed().map(|d| d.as_secs()).unwrap_or(u64::MAX))
}

/// 删除会话：移到系统废纸篓（可恢复）。
/// 路径归属校验复用 resume::parse_target（已知会话目录下的 .jsonl）；
/// 进行中判定由服务端按 mtime 独立执行，不信任前端传来的状态
pub fn delete(file_path: &str) -> Result<(), String> {
    crate::resume::parse_target(file_path)?;
    let path = Path::new(file_path)
        .canonicalize()
        .map_err(|_| "会话文件不存在")?;
    let age = file_age(&path).ok_or("无法读取文件状态")?;
    if age < RUNNING_SECS {
        return Err("会话正在进行中，不能删除".into());
    }
    trash::delete(&path).map_err(|_| "移到废纸篓失败".to_string())
}

/// 分页列出历史会话；keyword 匹配标题或项目路径（不区分大小写），
/// provider 为 "claude" / "codex" / None（全部）
pub fn list(offset: usize, limit: usize, keyword: Option<&str>, provider: Option<&str>) -> Vec<HistorySession> {
    let keyword = keyword.map(str::to_lowercase).filter(|k| !k.is_empty());
    let mut skipped = 0;
    let mut out = Vec::new();

    for (path, age, prov) in all_session_files() {
        if out.len() >= limit {
            break;
        }
        if let Some(p) = provider {
            if p != prov {
                continue;
            }
        }
        let Some(head) = parse_head(&path, prov) else {
            continue;
        };
        // 隐藏目录中的会话（插件后台 agent）不是用户自己的任务
        if head.project_path.contains("/.") {
            continue;
        }
        if let Some(k) = &keyword {
            if !head.title.to_lowercase().contains(k)
                && !head.project_path.to_lowercase().contains(k)
            {
                continue;
            }
        }
        if skipped < offset {
            skipped += 1;
            continue;
        }
        out.push(HistorySession {
            file_path: path.to_string_lossy().into_owned(),
            provider: prov.to_string(),
            title: head.title,
            project_path: head.project_path,
            last_activity_at: now_ms() - (age as i64) * 1000,
            is_active: age < RUNNING_SECS,
        });
    }
    out
}

struct Head {
    title: String,
    project_path: String,
}

/// 只读文件头部，取标题与项目目录
fn parse_head(path: &Path, provider: &str) -> Option<Head> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut ai_title: Option<String> = None;
    let mut summary: Option<String> = None;
    let mut first_user_text: Option<String> = None;
    let mut cwd: Option<String> = None;

    for line in reader.lines().take(HEAD_LINES).flatten() {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match provider {
            "claude" => {
                if v.get("isSidechain").and_then(Value::as_bool) == Some(true)
                    || v.get("isMeta").and_then(Value::as_bool) == Some(true)
                {
                    continue;
                }
                if cwd.is_none() {
                    if let Some(c) = v.get("cwd").and_then(Value::as_str) {
                        cwd = Some(c.to_string());
                    }
                }
                match v.get("type").and_then(Value::as_str) {
                    Some("ai-title") => {
                        ai_title = v
                            .get("aiTitle")
                            .and_then(Value::as_str)
                            .map(String::from);
                    }
                    Some("summary") => {
                        if summary.is_none() {
                            summary = v
                                .get("summary")
                                .and_then(Value::as_str)
                                .map(String::from);
                        }
                    }
                    Some("user") => {
                        if first_user_text.is_none() {
                            if let Some(text) = v
                                .get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(Value::as_str)
                            {
                                if !text.starts_with('<') {
                                    first_user_text = Some(truncate_chars(text, 60));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                // codex：session_meta 带 cwd，首条 user_message 当标题
                if cwd.is_none() {
                    if let Some(c) = v
                        .get("payload")
                        .and_then(|p| p.get("cwd"))
                        .and_then(Value::as_str)
                    {
                        cwd = Some(c.to_string());
                    }
                }
                if first_user_text.is_none()
                    && v.get("type").and_then(Value::as_str) == Some("event_msg")
                {
                    let payload = v.get("payload").unwrap_or(&Value::Null);
                    if payload.get("type").and_then(Value::as_str) == Some("user_message") {
                        if let Some(text) = payload.get("message").and_then(Value::as_str) {
                            if !text.starts_with('<') {
                                first_user_text = Some(truncate_chars(text, 60));
                            }
                        }
                    }
                }
            }
        }
        if ai_title.is_some() && cwd.is_some() {
            break;
        }
    }

    let project_path = cwd.unwrap_or_default();
    let title = ai_title
        .or(summary)
        .or(first_user_text)
        .unwrap_or_else(|| super::claude::project_name(&project_path));
    Some(Head {
        title,
        project_path,
    })
}

/// 提取单个会话的对话消息（最近 max 条）。file_path 必须通过数据源目录校验。
pub fn messages(file_path: &str, max: usize) -> Result<Vec<ChatMessage>, String> {
    let (provider, _) = crate::resume::parse_target(file_path)?;
    let file = fs::File::open(file_path).map_err(|_| "无法读取会话文件")?;
    let reader = BufReader::new(file);

    let mut out: Vec<ChatMessage> = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match provider {
            "claude" => collect_claude(&v, &mut out),
            _ => collect_codex(&v, &mut out),
        }
    }
    if out.len() > max {
        out.drain(..out.len() - max);
    }
    Ok(out)
}

fn collect_claude(v: &Value, out: &mut Vec<ChatMessage>) {
    if v.get("isSidechain").and_then(Value::as_bool) == Some(true)
        || v.get("isMeta").and_then(Value::as_bool) == Some(true)
    {
        return;
    }
    if v.get("message")
        .and_then(|m| m.get("model"))
        .and_then(Value::as_str)
        == Some("<synthetic>")
    {
        return;
    }
    let at = v
        .get("timestamp")
        .and_then(Value::as_str)
        .map(String::from);
    match v.get("type").and_then(Value::as_str) {
        Some("user") => {
            let content = v.get("message").and_then(|m| m.get("content"));
            // 字符串形式是用户输入；数组形式多为 tool_result，跳过
            if let Some(text) = content.and_then(Value::as_str) {
                if !text.starts_with('<') && !text.trim().is_empty() {
                    out.push(ChatMessage {
                        role: "user".into(),
                        text: truncate_chars(text, MSG_MAX_CHARS),
                        at,
                    });
                }
            }
        }
        Some("assistant") => {
            let Some(blocks) = v
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(Value::as_array)
            else {
                return;
            };
            for block in blocks {
                match block.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            if !text.trim().is_empty() {
                                out.push(ChatMessage {
                                    role: "assistant".into(),
                                    text: truncate_chars(text, MSG_MAX_CHARS),
                                    at: at.clone(),
                                });
                            }
                        }
                    }
                    Some("tool_use") => {
                        let name = block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("(工具)");
                        let input = block.get("input").unwrap_or(&Value::Null);
                        out.push(ChatMessage {
                            role: "tool".into(),
                            text: super::claude::describe_tool(name, input),
                            at: at.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn collect_codex(v: &Value, out: &mut Vec<ChatMessage>) {
    let at = v
        .get("timestamp")
        .and_then(Value::as_str)
        .map(String::from);
    let payload = v.get("payload").unwrap_or(&Value::Null);
    match v.get("type").and_then(Value::as_str) {
        Some("event_msg") => match payload.get("type").and_then(Value::as_str) {
            Some("user_message") => {
                if let Some(text) = payload.get("message").and_then(Value::as_str) {
                    if !text.starts_with('<') && !text.trim().is_empty() {
                        out.push(ChatMessage {
                            role: "user".into(),
                            text: truncate_chars(text, MSG_MAX_CHARS),
                            at,
                        });
                    }
                }
            }
            Some("agent_message") => {
                if let Some(text) = payload.get("message").and_then(Value::as_str) {
                    out.push(ChatMessage {
                        role: "assistant".into(),
                        text: truncate_chars(text, MSG_MAX_CHARS),
                        at,
                    });
                }
            }
            _ => {}
        },
        Some("response_item") => {
            if payload.get("type").and_then(Value::as_str) == Some("function_call") {
                let name = payload
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("(工具)");
                out.push(ChatMessage {
                    role: "tool".into(),
                    text: name.to_string(),
                    at,
                });
            }
        }
        _ => {}
    }
}
