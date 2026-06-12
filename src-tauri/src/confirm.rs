//! 面板内确认：Claude Code 的 PreToolUse hook 通过本机 unix socket 询问面板，
//! 用户在面板点「允许/拒绝」后把决定返回给 hook。
//!
//! 安全约定（见 docs/plans/2026-06-11-panel-confirm.md）：
//! - socket 目录 0700、socket 0600，仅本用户进程可连，不开任何 TCP 端口
//! - 请求中的 transcript_path 必须真实存在且位于 ~/.claude/projects 下
//! - 任何异常/超时一律返回空（回落到终端提示），绝不缺省放行
//! - 工具参数仅在内存流转，不落盘、不写日志

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::Manager;

use crate::models::{now_ms, truncate_chars};

/// 服务端等待面板决定的上限；hook 侧 nc 超时 50s、settings 中 hook timeout 55s，逐层兜底
const DECIDE_TIMEOUT_SECS: u64 = 45;
/// 拦截的工具（与 hook 安装时写入的 matcher 保持一致）
const HOOK_MATCHER: &str = "Bash|Write|Edit|MultiEdit|NotebookEdit";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmInfo {
    pub id: u64,
    pub tool_name: String,
    pub summary: String,
    pub project_path: String,
    /// 会话 jsonl 路径，用于和会话卡片关联
    pub transcript_path: String,
    /// epoch 毫秒
    pub created_at: i64,
}

struct PendingEntry {
    info: ConfirmInfo,
    decide: SyncSender<String>,
}

fn pending() -> &'static Mutex<HashMap<u64, PendingEntry>> {
    static PENDING: OnceLock<Mutex<HashMap<u64, PendingEntry>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

fn panel_dir() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".cc_panel"))
}

fn socket_path() -> Option<PathBuf> {
    Some(panel_dir()?.join("confirm.sock"))
}

fn hook_script_path() -> Option<PathBuf> {
    Some(panel_dir()?.join("confirm-hook.sh"))
}

/// 当前待确认列表（给前端轮询）
pub fn list() -> Vec<ConfirmInfo> {
    let Ok(map) = pending().lock() else {
        return Vec::new();
    };
    let mut items: Vec<ConfirmInfo> = map.values().map(|e| e.info.clone()).collect();
    items.sort_by_key(|i| i.id);
    items
}

/// 用户在面板点了允许/拒绝。返回 false 表示该请求已超时/不存在
pub fn resolve(id: u64, decision: &str) -> bool {
    if decision != "allow" && decision != "deny" {
        return false;
    }
    let Ok(map) = pending().lock() else {
        return false;
    };
    match map.get(&id) {
        Some(entry) => entry.decide.send(decision.to_string()).is_ok(),
        None => false,
    }
}

pub fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        if let Err(e) = run_server(app) {
            eprintln!("confirm socket 服务退出: {e}");
        }
    });
}

fn run_server(app: tauri::AppHandle) -> std::io::Result<()> {
    let Some(dir) = panel_dir() else {
        return Ok(());
    };
    std::fs::create_dir_all(&dir)?;
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    let Some(sock) = socket_path() else {
        return Ok(());
    };
    let _ = std::fs::remove_file(&sock); // 清掉上次异常退出留下的 socket
    let listener = UnixListener::bind(&sock)?;
    std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600))?;

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let app = app.clone();
        std::thread::spawn(move || handle_conn(stream, app));
    }
    Ok(())
}

fn handle_conn(stream: UnixStream, app: tauri::AppHandle) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return reply(stream, "");
    }
    let Ok(req) = serde_json::from_str::<Value>(line.trim()) else {
        return reply(stream, "");
    };

    // 校验请求来自真实的 Claude 会话，防伪造请求骗取 allow
    let transcript = req
        .get("transcript_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !transcript_valid(transcript) {
        return reply(stream, "");
    }

    // bypassPermissions/plan 模式本就不会弹终端提示；acceptEdits 下编辑类工具自动放行
    let tool_name = req.get("tool_name").and_then(Value::as_str).unwrap_or("");
    let mode = req
        .get("permission_mode")
        .and_then(Value::as_str)
        .unwrap_or("default");
    let is_edit_tool = matches!(tool_name, "Write" | "Edit" | "MultiEdit" | "NotebookEdit");
    if mode == "bypassPermissions" || mode == "plan" || (mode == "acceptEdits" && is_edit_tool) {
        return reply(stream, "");
    }

    // 命中用户 allowlist 的 Bash 命令会被 Claude Code 自动放行，不值得打扰
    let tool_input = req.get("tool_input").cloned().unwrap_or(Value::Null);
    let cwd = req.get("cwd").and_then(Value::as_str).unwrap_or("");
    if allowlisted(tool_name, &tool_input, cwd) {
        return reply(stream, "");
    }

    // jsonl 链路对同一次停顿的启发式通知会用这个 key（文案逐字一致），
    // hook 已提醒过的停顿据此压制，不同确认点互不影响
    let jsonl_key = format!(
        "waiting:{transcript}:可能在等待权限确认 — {}",
        crate::sources::claude::describe_tool(tool_name, &tool_input)
    );

    // 面板窗口不可见时不拦截：用户在终端工作，走原生提示零延迟。
    // 但确认请求本身是准确信号，补发系统通知防止挂起无人知晓
    let visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if !visible {
        if crate::load_panel_settings().notify_confirm {
            crate::notify_dedup(
                &app,
                jsonl_key,
                "等待权限确认（CC Panel）",
                &format!("{}: {}", tool_name, summarize(tool_name, &tool_input)),
                Some(transcript),
            );
        }
        return reply(stream, "");
    }

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = sync_channel::<String>(1);
    let info = ConfirmInfo {
        id,
        tool_name: tool_name.to_string(),
        summary: summarize(tool_name, &tool_input),
        project_path: cwd.to_string(),
        transcript_path: transcript.to_string(),
        created_at: now_ms(),
    };
    let notify_body = format!("{}: {}", info.tool_name, info.summary);
    if let Ok(mut map) = pending().lock() {
        map.insert(id, PendingEntry { info, decide: tx });
    } else {
        return reply(stream, "");
    }

    if crate::load_panel_settings().notify_confirm {
        crate::notify_dedup(
            &app,
            format!("confirm:{id}"),
            "待确认（CC Panel）",
            &notify_body,
            Some(transcript),
        );
        // 45s 超时回落终端后 jsonl 启发式可能再次命中同一停顿，预先压制
        crate::notify_mark(jsonl_key);
    }

    let decision = rx.recv_timeout(Duration::from_secs(DECIDE_TIMEOUT_SECS)).ok();
    if let Ok(mut map) = pending().lock() {
        map.remove(&id);
    }

    match decision.as_deref() {
        Some(d @ ("allow" | "deny")) => {
            let reason = if d == "allow" {
                "已在 CC Panel 批准"
            } else {
                "已在 CC Panel 拒绝"
            };
            let out = json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": d,
                    "permissionDecisionReason": reason,
                }
            });
            reply(stream, &out.to_string());
        }
        // 超时或异常：返回空，回落到终端原生提示
        _ => reply(stream, ""),
    }
}

fn reply(mut stream: UnixStream, body: &str) {
    if !body.is_empty() {
        let _ = stream.write_all(body.as_bytes());
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn transcript_valid(path: &str) -> bool {
    let Some(projects) = crate::sources::claude::projects_dir() else {
        return false;
    };
    let p = std::path::Path::new(path);
    p.is_absolute() && p.starts_with(&projects) && p.exists()
}

/// 工具参数摘要（仅展示用，截断防泄露大段内容）
fn summarize(tool_name: &str, input: &Value) -> String {
    let text = match tool_name {
        "Bash" => input
            .get("command")
            .and_then(Value::as_str)
            .map(String::from),
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => input
            .get("file_path")
            .or_else(|| input.get("notebook_path"))
            .and_then(Value::as_str)
            .map(String::from),
        _ => None,
    };
    truncate_chars(&text.unwrap_or_default(), 140)
}

/// 是否命中 Claude Code 的 allow 规则（全局 + 项目 + 项目 local 三层）。
/// 只复刻 allow 方向：误判的代价仅是确认入口从面板换回终端，不存在放行风险。
fn allowlisted(tool_name: &str, input: &Value, cwd: &str) -> bool {
    let mut rules: Vec<String> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();
    if let Some(home) = dirs::home_dir() {
        files.push(home.join(".claude").join("settings.json"));
    }
    if !cwd.is_empty() {
        files.push(PathBuf::from(cwd).join(".claude").join("settings.json"));
        files.push(PathBuf::from(cwd).join(".claude").join("settings.local.json"));
    }
    for f in files {
        let Ok(raw) = std::fs::read_to_string(f) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if let Some(allow) = v
            .get("permissions")
            .and_then(|p| p.get("allow"))
            .and_then(Value::as_array)
        {
            rules.extend(allow.iter().filter_map(Value::as_str).map(String::from));
        }
    }

    let command = input.get("command").and_then(Value::as_str).unwrap_or("");
    for rule in &rules {
        // 整个工具被放行，如 "WebFetch"
        if rule == tool_name {
            return true;
        }
        if tool_name == "Bash" && !command.is_empty() {
            if let Some(body) = rule
                .strip_prefix("Bash(")
                .and_then(|r| r.strip_suffix(')'))
            {
                if let Some(prefix) = body.strip_suffix(":*") {
                    if command.trim_start().starts_with(prefix) {
                        return true;
                    }
                } else if command.trim() == body {
                    return true;
                }
            }
        }
    }
    false
}

// ---------- hook 安装/卸载（由用户在面板设置中显式触发） ----------

const HOOK_SCRIPT: &str = r#"#!/bin/bash
# CC Panel 面板内确认 hook（由 CC Panel 设置页生成/移除）
# 面板未运行或不可达时输出空并退出，Claude Code 走原生确认流程
SOCK="$HOME/.cc_panel/confirm.sock"
[ -S "$SOCK" ] || exit 0
RESP=$({ cat; echo; } | /usr/bin/nc -U "$SOCK" -w 50 2>/dev/null)
[ -n "$RESP" ] && printf '%s' "$RESP"
exit 0
"#;

fn claude_settings_path() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".claude").join("settings.json"))
}

fn is_our_hook_entry(entry: &Value) -> bool {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .map(|hs| {
            hs.iter().any(|h| {
                h.get("command")
                    .and_then(Value::as_str)
                    .is_some_and(|c| c.contains(".cc_panel/confirm-hook"))
            })
        })
        .unwrap_or(false)
}

/// hook 当前是否已安装
pub fn hook_installed() -> bool {
    let Some(path) = claude_settings_path() else {
        return false;
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    v.get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().any(is_our_hook_entry))
        .unwrap_or(false)
}

/// 安装/卸载 hook。写入前备份 settings.json 到 settings.json.cc-panel.bak
pub fn set_hook(enabled: bool) -> Result<bool, String> {
    let script = hook_script_path().ok_or("无法定位用户目录")?;
    let settings_path = claude_settings_path().ok_or("无法定位 ~/.claude")?;

    if enabled {
        let dir = panel_dir().ok_or("无法定位用户目录")?;
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建 ~/.cc_panel 失败: {e}"))?;
        std::fs::write(&script, HOOK_SCRIPT).map_err(|e| format!("写入 hook 脚本失败: {e}"))?;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置脚本权限失败: {e}"))?;
    }

    let raw = std::fs::read_to_string(&settings_path).unwrap_or_else(|_| "{}".into());
    let mut settings: Value =
        serde_json::from_str(&raw).map_err(|_| "settings.json 不是合法 JSON，已中止以免损坏")?;

    // 备份原文件（仅当存在时）
    if settings_path.exists() {
        let backup = settings_path.with_extension("json.cc-panel.bak");
        std::fs::copy(&settings_path, &backup).map_err(|e| format!("备份失败，已中止: {e}"))?;
    }

    let hooks = settings
        .as_object_mut()
        .ok_or("settings.json 顶层不是对象")?
        .entry("hooks")
        .or_insert_with(|| json!({}));
    let pre = hooks
        .as_object_mut()
        .ok_or("hooks 字段不是对象")?
        .entry("PreToolUse")
        .or_insert_with(|| json!([]));
    let arr = pre.as_array_mut().ok_or("hooks.PreToolUse 不是数组")?;

    arr.retain(|e| !is_our_hook_entry(e)); // 幂等：先移除旧条目
    if enabled {
        arr.push(json!({
            "matcher": HOOK_MATCHER,
            "hooks": [{
                "type": "command",
                "command": script.to_string_lossy(),
                "timeout": 55,
            }]
        }));
    }

    let pretty =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&settings_path, pretty).map_err(|e| format!("写入 settings.json 失败: {e}"))?;
    Ok(enabled)
}
