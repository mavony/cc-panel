//! 解析 Codex 本地会话（~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl）
//! 额度信息直接来自会话内 token_count 事件的 rate_limits 字段，无需网络请求。

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::models::{
    now_ms, status_for_age, truncate_chars, ProviderUsage, SessionDetail, SessionSummary,
    TodoItem, UsageWindow,
};

pub fn sessions_dir() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".codex").join("sessions"))
}

/// Codex CLI 是否安装（~/.codex 存在即认为安装过）
pub fn installed() -> bool {
    dirs::home_dir().is_some_and(|h| h.join(".codex").exists())
}

fn config_lines() -> Vec<String> {
    let Some(path) = dirs::home_dir().map(|h| h.join(".codex").join("config.toml")) else {
        return Vec::new();
    };
    fs::read_to_string(path)
        .map(|s| s.lines().map(String::from).collect())
        .unwrap_or_default()
}

/// 取 config.toml 顶层（首个 [section] 之前）的字符串配置项
fn config_top_level(key: &str) -> Option<String> {
    for line in config_lines() {
        let line = line.trim();
        if line.starts_with('[') {
            break;
        }
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                return Some(value.trim().trim_matches('"').to_string()).filter(|v| !v.is_empty());
            }
        }
    }
    None
}

/// 当前模型：优先 config.toml 顶层 model，缺省时取最新会话 turn_context 的 model
pub fn current_model() -> Option<String> {
    if let Some(m) = config_top_level("model") {
        return Some(m);
    }
    let mut files = recent_files(7 * 24 * 3600);
    files.sort_by_key(|(_, age)| *age);
    let (path, _) = files.into_iter().next()?;
    crate::sources::claude_usage::session_file_model(&path)
}

/// 设置了非默认 model_provider 即视为第三方代理，取该 provider 的 base_url host
pub fn proxy_host() -> Option<String> {
    let provider = config_top_level("model_provider")?;
    if provider == "openai" {
        return None;
    }
    let section = format!("[model_providers.{provider}]");
    let mut in_section = false;
    for line in config_lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if in_section {
            if let Some(rest) = trimmed.strip_prefix("base_url") {
                if let Some(value) = rest.trim_start().strip_prefix('=') {
                    let url = value.trim().trim_matches('"');
                    let no_scheme = url.split("://").nth(1).unwrap_or(url);
                    let host = no_scheme.split(['/', '?', '#']).next().unwrap_or(no_scheme);
                    return Some(host.to_string()).filter(|h| !h.is_empty());
                }
            }
        }
    }
    // 配了非默认 provider 但找不到 base_url，仍按代理处理
    Some(provider)
}

pub(crate) fn recent_files(max_age_secs: u64) -> Vec<(PathBuf, u64)> {
    let Some(root) = sessions_dir() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect_jsonl(&root, 0, max_age_secs, &mut out);
    out
}

fn collect_jsonl(dir: &Path, depth: u8, max_age_secs: u64, out: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < 4 {
                collect_jsonl(&path, depth + 1, max_age_secs, out);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            let Ok(meta) = entry.metadata() else { continue };
            let Ok(modified) = meta.modified() else { continue };
            let age = modified.elapsed().map(|d| d.as_secs()).unwrap_or(u64::MAX);
            if age < max_age_secs {
                out.push((path, age));
            }
        }
    }
}

pub fn scan_sessions() -> Vec<SessionDetail> {
    recent_files(crate::models::RECENT_SECS)
        .into_iter()
        .filter_map(|(path, age)| {
            let status = status_for_age(age)?;
            parse_session(&path, status, age)
        })
        .collect()
}

fn parse_session(path: &Path, status: &str, age_secs: u64) -> Option<SessionDetail> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut cwd = String::new();
    let mut started_at: Option<String> = None;
    let mut title: Option<String> = None;
    let mut message_count: u32 = 0;
    let mut latest_message = String::new();
    let mut current_activity = String::new();
    let mut todos: Vec<TodoItem> = Vec::new();

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let payload = v.get("payload").unwrap_or(&Value::Null);
        match v.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                if let Some(c) = payload.get("cwd").and_then(Value::as_str) {
                    cwd = c.to_string();
                }
                if let Some(ts) = payload.get("timestamp").and_then(Value::as_str) {
                    started_at = Some(ts.to_string());
                }
            }
            Some("event_msg") => match payload.get("type").and_then(Value::as_str) {
                Some("user_message") => {
                    message_count += 1;
                    if title.is_none() {
                        if let Some(text) = payload.get("message").and_then(Value::as_str) {
                            if !text.starts_with('<') {
                                title = Some(truncate_chars(text, 40));
                            }
                        }
                    }
                }
                Some("agent_message") => {
                    message_count += 1;
                    if let Some(text) = payload.get("message").and_then(Value::as_str) {
                        latest_message = truncate_chars(text, 200);
                        current_activity = "正在回复".into();
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
                    current_activity = name.to_string();
                    if name == "update_plan" {
                        if let Some(parsed) = payload
                            .get("arguments")
                            .and_then(Value::as_str)
                            .and_then(|s| serde_json::from_str::<Value>(s).ok())
                        {
                            if let Some(plan) = parsed.get("plan").and_then(Value::as_array) {
                                todos = plan
                                    .iter()
                                    .map(|p| TodoItem {
                                        content: p
                                            .get("step")
                                            .and_then(Value::as_str)
                                            .unwrap_or("")
                                            .to_string(),
                                        status: p
                                            .get("status")
                                            .and_then(Value::as_str)
                                            .unwrap_or("pending")
                                            .to_string(),
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let title = title.unwrap_or_else(|| crate::sources::claude::project_name(&cwd));
    let summary = SessionSummary {
        file_path: path.to_string_lossy().into_owned(),
        provider: "codex".into(),
        title,
        project_path: cwd,
        status: status.to_string(),
        started_at,
        last_activity_at: now_ms() - (age_secs as i64) * 1000,
        message_count,
        current_activity,
        pending_question: None,
    };
    Some(SessionDetail {
        summary,
        todos,
        outputs: Vec::new(),
        latest_message,
        git_branch: None,
    })
}

/// 从最近会话文件中找最新的 rate_limits（token_count 事件随每轮响应写入）
pub fn usage() -> ProviderUsage {
    let model = current_model();

    // 第三方代理：rate_limits 来自代理转发，额度数字不具参考性，如实降级展示
    if let Some(host) = proxy_host() {
        return ProviderUsage {
            provider: "codex".into(),
            ok: true,
            plan: None,
            windows: Vec::new(),
            fetched_at: now_ms(),
            error: None,
            model,
            source: "proxy".into(),
            proxy_host: Some(host),
        };
    }

    // 回看 7 天，额度窗口本身是 5h/30d 级别的
    let mut files = recent_files(7 * 24 * 3600);
    files.sort_by_key(|(_, age)| *age);

    for (path, _) in files {
        if let Some(mut u) = latest_rate_limits(&path) {
            u.model = model;
            return u;
        }
    }
    ProviderUsage {
        provider: "codex".into(),
        ok: false,
        plan: None,
        windows: Vec::new(),
        fetched_at: now_ms(),
        error: Some("最近 7 天没有 Codex 会话记录，无额度数据".into()),
        model,
        source: "official".into(),
        proxy_host: None,
    }
}

fn latest_rate_limits(path: &Path) -> Option<ProviderUsage> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut latest: Option<Value> = None;
    for line in reader.lines() {
        let Ok(line) = line else { continue };
        // 粗筛避免逐行完整解析
        if !line.contains("rate_limits") {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            if let Some(rl) = v.get("payload").and_then(|p| p.get("rate_limits")) {
                if !rl.is_null() {
                    latest = Some(rl.clone());
                }
            }
        }
    }
    let rl = latest?;
    let mut windows = Vec::new();
    for (key, label) in [("primary", "5 小时"), ("secondary", "周期")] {
        let Some(w) = rl.get(key).filter(|w| !w.is_null()) else {
            continue;
        };
        let minutes = w.get("window_minutes").and_then(Value::as_i64).unwrap_or(0);
        let label = match minutes {
            0 => label.to_string(),
            m if m % (24 * 60) == 0 => format!("{} 天", m / (24 * 60)),
            m if m % 60 == 0 => format!("{} 小时", m / 60),
            m => format!("{m} 分钟"),
        };
        windows.push(UsageWindow {
            label,
            used_percent: w.get("used_percent").and_then(Value::as_f64).unwrap_or(0.0),
            resets_at: w.get("resets_at").and_then(Value::as_i64),
        });
    }
    Some(ProviderUsage {
        provider: "codex".into(),
        ok: !windows.is_empty(),
        plan: rl
            .get("plan_type")
            .and_then(Value::as_str)
            .map(String::from),
        windows,
        fetched_at: now_ms(),
        error: None,
        model: None,
        source: "official".into(),
        proxy_host: None,
    })
}
