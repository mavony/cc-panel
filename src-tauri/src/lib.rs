mod confirm;
mod models;
mod sources;

use models::{ProviderUsage, SessionDetail};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

/// 列出两个数据源里最近活跃的会话（含展开详情），进行中的排前面
#[tauri::command]
fn list_sessions() -> Vec<SessionDetail> {
    let mut sessions = sources::claude::scan_sessions();
    sessions.extend(sources::codex::scan_sessions());
    sessions.sort_by(|a, b| {
        let rank = |s: &SessionDetail| match s.summary.status.as_str() {
            "waiting" => 0,
            "running" => 1,
            _ => 2,
        };
        rank(a)
            .cmp(&rank(b))
            .then(b.summary.last_activity_at.cmp(&a.summary.last_activity_at))
    });
    sessions
}

/// 只返回本机实际安装的 CLI 的额度卡片
#[tauri::command]
async fn get_usage() -> Vec<ProviderUsage> {
    let mut usages = Vec::new();
    if sources::claude_usage::installed() {
        usages.push(sources::claude_usage::usage().await);
    }
    if sources::codex::installed() {
        let codex = tauri::async_runtime::spawn_blocking(sources::codex::usage)
            .await
            .unwrap_or_else(|_| ProviderUsage {
                provider: "codex".into(),
                ok: false,
                plan: None,
                windows: Vec::new(),
                fetched_at: models::now_ms(),
                error: Some("Codex 额度扫描失败".into()),
                model: None,
                source: "official".into(),
                proxy_host: None,
            });
        usages.push(codex);
    }
    usages
}

/// 面板自有设置（~/.cc_panel/settings.json，与 ~/.claude 无关）
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct PanelSettings {
    /// 出现待确认时发系统通知
    pub notify_confirm: bool,
    /// 会话结束时发系统通知
    pub notify_done: bool,
}

impl Default for PanelSettings {
    fn default() -> Self {
        Self {
            notify_confirm: true,
            notify_done: true,
        }
    }
}

fn panel_settings_path() -> Option<std::path::PathBuf> {
    Some(dirs::home_dir()?.join(".cc_panel").join("settings.json"))
}

pub fn load_panel_settings() -> PanelSettings {
    panel_settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn get_panel_settings() -> PanelSettings {
    load_panel_settings()
}

#[tauri::command]
fn set_panel_settings(settings: PanelSettings) -> Result<(), String> {
    let path = panel_settings_path().ok_or("无法定位用户目录")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(path, raw).map_err(|e| format!("写入设置失败: {e}"))
}

/// 最近一次通知关联的会话（用于点通知后聚焦展开；10 分钟内有效，取用即清空）
fn last_notified() -> &'static std::sync::Mutex<Option<(String, std::time::Instant)>> {
    static LAST: std::sync::OnceLock<std::sync::Mutex<Option<(String, std::time::Instant)>>> =
        std::sync::OnceLock::new();
    LAST.get_or_init(|| std::sync::Mutex::new(None))
}

fn take_recent_notified() -> Option<String> {
    let mut guard = last_notified().lock().ok()?;
    let (path, at) = guard.take()?;
    if at.elapsed() < std::time::Duration::from_secs(600) {
        Some(path)
    } else {
        None
    }
}

/// 把"最近通知的会话"推给前端聚焦展开（无待聚焦会话时为空操作）
fn emit_focus_session(app: &tauri::AppHandle) {
    if let Some(path) = take_recent_notified() {
        use tauri::Emitter;
        let _ = app.emit("focus-session", path);
    }
}

/// 发系统通知（带系统提示音）。同一 key 在 10 分钟内去重；
/// session 为通知关联的会话 jsonl 路径，用于点击通知后的聚焦联动。
pub fn notify_dedup(
    app: &tauri::AppHandle,
    key: String,
    title: &str,
    body: &str,
    session: Option<&str>,
) {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    static LAST: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    let last = LAST.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut map) = last.lock() {
        let now = Instant::now();
        if let Some(t) = map.get(&key) {
            if now.duration_since(*t) < Duration::from_secs(600) {
                return;
            }
        }
        map.retain(|_, t| now.duration_since(*t) < Duration::from_secs(600));
        map.insert(key, now);
    }

    if let Some(path) = session {
        if let Ok(mut guard) = last_notified().lock() {
            *guard = Some((path.to_string(), std::time::Instant::now()));
        }
    }

    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .sound("Glass")
        .show();
}

/// 待确认的工具权限请求列表（来自 PreToolUse hook）
#[tauri::command]
fn list_pending_confirms() -> Vec<confirm::ConfirmInfo> {
    confirm::list()
}

/// 用户在面板对某个权限请求做出决定（allow/deny）
#[tauri::command]
fn resolve_confirm(id: u64, decision: String) -> bool {
    confirm::resolve(id, &decision)
}

/// 面板内确认 hook 是否已安装到 ~/.claude/settings.json
#[tauri::command]
fn confirm_hook_status() -> bool {
    confirm::hook_installed()
}

/// 安装/卸载面板内确认 hook（用户在设置中显式触发）
#[tauri::command]
fn set_confirm_hook(enabled: bool) -> Result<bool, String> {
    confirm::set_hook(enabled)
}

/// 在 Finder 中显示文件/目录。只接受真实存在的绝对路径。
#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.is_absolute() || !p.exists() {
        return Err("路径不存在".into());
    }
    tauri_plugin_opener::reveal_item_in_dir(p).map_err(|_| "无法在 Finder 中打开".into())
}

/// 以下 debug_* 函数仅供 examples/probe.rs 命令行验证使用
pub fn debug_scan_claude() -> Vec<serde_json::Value> {
    sources::claude::scan_sessions()
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect()
}

pub fn debug_scan_codex() -> Vec<serde_json::Value> {
    sources::codex::scan_sessions()
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect()
}

pub fn debug_codex_usage() -> serde_json::Value {
    serde_json::to_value(sources::codex::usage()).unwrap_or_default()
}

/// 后台循环：刷新 tray 计数 + 状态变化通知（新 waiting / running→recent）
fn watch_sessions(handle: tauri::AppHandle) {
    let mut prev: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    loop {
        let claude = sources::claude::scan_sessions();
        let codex = sources::codex::scan_sessions();
        let sessions: Vec<_> = claude.iter().chain(codex.iter()).collect();

        let running = sessions.iter().filter(|s| s.summary.status == "running").count();
        let waiting = sessions.iter().filter(|s| s.summary.status == "waiting").count();
        if let Some(tray) = handle.tray_by_id("main") {
            let mut parts = Vec::new();
            if running > 0 {
                parts.push(format!("▶ {running}"));
            }
            if waiting > 0 {
                parts.push(format!("⏸ {waiting}"));
            }
            let title = if parts.is_empty() {
                None
            } else {
                Some(parts.join(" "))
            };
            let _ = tray.set_title(title);
        }

        let settings = load_panel_settings();
        for s in &sessions {
            let path = &s.summary.file_path;
            let status = s.summary.status.as_str();
            let old = prev.get(path).map(String::as_str);
            // 新进入 waiting：会话停在 AskUserQuestion / 计划批准上
            if settings.notify_confirm && status == "waiting" && old != Some("waiting") {
                let q = s.summary.pending_question.as_deref().unwrap_or("等待确认");
                notify_dedup(
                    &handle,
                    format!("waiting:{path}"),
                    &format!("等待确认：{}", s.summary.title),
                    q,
                    Some(path),
                );
            }
            // running → recent：会话静默约 2 分钟，视为结束
            if settings.notify_done && status == "recent" && old == Some("running") {
                notify_dedup(
                    &handle,
                    format!("done:{path}"),
                    "会话结束",
                    &s.summary.title,
                    Some(path),
                );
            }
        }
        prev = sessions
            .iter()
            .map(|s| (s.summary.file_path.clone(), s.summary.status.clone()))
            .collect();

        std::thread::sleep(std::time::Duration::from_secs(15));
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_usage,
            reveal_path,
            list_pending_confirms,
            resolve_confirm,
            confirm_hook_status,
            set_confirm_hook,
            get_panel_settings,
            set_panel_settings
        ])
        .setup(|app| {
            confirm::start(app.handle().clone());

            let show = MenuItem::with_id(app, "show", "显示面板", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        show_main_window(app);
                        emit_focus_session(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // 后台线程定期刷新 tray 标题：显示进行中的会话数
            let handle = app.handle().clone();
            std::thread::spawn(move || watch_sessions(handle));
            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::Manager as _;
            match event {
                // 关窗只隐藏，应用常驻 tray
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                // 窗口获焦（含点通知激活应用后回到面板）时联动聚焦最近通知的会话
                tauri::WindowEvent::Focused(true) => {
                    emit_focus_session(window.app_handle());
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // 点通知/点 Dock 激活应用且无可见窗口时（macOS Reopen），亮出面板并聚焦会话
            if let tauri::RunEvent::Reopen { .. } = event {
                show_main_window(app);
                emit_focus_session(app);
            }
        });
}
