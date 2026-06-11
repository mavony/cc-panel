//! Claude 订阅额度：读取 Claude Code 的 OAuth token，调用 usage 端点。
//! 注意：该端点非官方公开接口，解析必须全程容错；token 只在本模块内存中使用，
//! 不写日志、不返回给前端、不落盘。

use std::process::Command;

use serde_json::Value;

use crate::models::{now_ms, ProviderUsage, UsageWindow};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

/// Claude Code 是否安装（~/.claude 存在即认为安装过）
pub fn installed() -> bool {
    dirs::home_dir().is_some_and(|h| h.join(".claude").exists())
}

fn settings() -> Option<Value> {
    let path = dirs::home_dir()?.join(".claude").join("settings.json");
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// settings.json 配置了 ANTHROPIC_BASE_URL 即视为走第三方代理，仅取 host
pub fn proxy_host() -> Option<String> {
    let url = settings()?
        .get("env")?
        .get("ANTHROPIC_BASE_URL")?
        .as_str()?
        .trim()
        .to_string();
    if url.is_empty() {
        return None;
    }
    Some(host_of(&url))
}

/// 从 URL 中提取 host（不引入 url crate 的轻量实现；含 key 的路径/参数一律丢弃）
fn host_of(url: &str) -> String {
    let no_scheme = url.split("://").nth(1).unwrap_or(url);
    no_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(no_scheme)
        .to_string()
}

/// 当前模型：优先 settings.json 的 model 字段；该字段是 "sonnet"/"opusplan" 这类
/// 不带版本号的别名时，改取最新会话里 assistant 消息的完整模型 ID（如 claude-sonnet-4-6）
pub fn current_model() -> Option<String> {
    let configured = settings()
        .and_then(|s| s.get("model").and_then(Value::as_str).map(String::from))
        .filter(|m| !m.is_empty());

    if let Some(m) = &configured {
        if m.chars().any(|c| c.is_ascii_digit()) {
            return configured;
        }
    }
    crate::sources::claude::projects_dir()
        .and_then(|root| latest_session_model(&root))
        .or(configured)
}

/// 在最新修改的会话 jsonl 中找最后一条 "model":"…"（粗筛行避免全量解析）
fn latest_session_model(root: &std::path::Path) -> Option<String> {
    let mut newest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    for project in std::fs::read_dir(root).ok()?.flatten() {
        let Ok(files) = std::fs::read_dir(project.path()) else {
            continue;
        };
        for f in files.flatten() {
            let path = f.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Ok(modified) = f.metadata().and_then(|m| m.modified()) else {
                continue;
            };
            if newest.as_ref().is_none_or(|(_, t)| modified > *t) {
                newest = Some((path, modified));
            }
        }
    }
    let (path, _) = newest?;
    session_file_model(&path)
}

pub(crate) fn session_file_model(path: &std::path::Path) -> Option<String> {
    use std::io::BufRead;
    let file = std::fs::File::open(path).ok()?;
    let mut model = None;
    for line in std::io::BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        if !line.contains("\"model\"") {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            // Claude 会话在 message.model，Codex 会话在 payload.model
            let m = v
                .get("message")
                .and_then(|m| m.get("model"))
                .or_else(|| v.get("payload").and_then(|p| p.get("model")))
                .or_else(|| v.get("model"))
                .and_then(Value::as_str);
            if let Some(m) = m {
                model = Some(m.to_string());
            }
        }
    }
    model
}

/// 先读 ~/.claude/.credentials.json，没有则从 macOS Keychain 读取
fn access_token() -> Option<String> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".claude").join(".credentials.json");
        if let Ok(raw) = std::fs::read_to_string(path) {
            if let Some(token) = token_from_credentials(&raw) {
                return Some(token);
            }
        }
    }
    let output = Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    token_from_credentials(&String::from_utf8_lossy(&output.stdout))
}

fn token_from_credentials(raw: &str) -> Option<String> {
    let v: Value = serde_json::from_str(raw.trim()).ok()?;
    v.get("claudeAiOauth")?
        .get("accessToken")?
        .as_str()
        .map(String::from)
}

pub async fn usage() -> ProviderUsage {
    let model = current_model();

    // 第三方代理：官方订阅额度与中转配额无关，不调官方端点，如实降级展示
    if let Some(host) = proxy_host() {
        return ProviderUsage {
            provider: "claude".into(),
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

    let fail = |msg: &str, model: Option<String>| ProviderUsage {
        provider: "claude".into(),
        ok: false,
        plan: None,
        windows: Vec::new(),
        fetched_at: now_ms(),
        error: Some(msg.to_string()),
        model,
        source: "official".into(),
        proxy_host: None,
    };

    let Some(token) = access_token() else {
        return fail("未找到 Claude Code 登录凭据", model);
    };

    let client = reqwest::Client::new();
    let resp = client
        .get(USAGE_URL)
        .bearer_auth(&token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(_) => return fail("额度接口请求失败（网络错误）", model),
    };
    if !resp.status().is_success() {
        return fail(&format!("额度接口返回 {}", resp.status().as_u16()), model);
    }
    let Ok(body) = resp.json::<Value>().await else {
        return fail("额度接口响应无法解析", model);
    };
    let mut usage = parse_usage(&body);
    usage.model = model;
    usage
}

/// 防御式解析：遍历顶层对象，凡是带 utilization 字段的对象都视为一个额度窗口
fn parse_usage(body: &Value) -> ProviderUsage {
    let mut windows = Vec::new();
    if let Some(obj) = body.as_object() {
        for (key, val) in obj {
            let Some(used) = val.get("utilization").and_then(Value::as_f64) else {
                continue;
            };
            let resets_at = val.get("resets_at").and_then(|r| {
                r.as_i64()
                    .or_else(|| r.as_str().and_then(parse_iso_to_epoch))
            });
            windows.push(UsageWindow {
                label: window_label(key),
                used_percent: used,
                resets_at,
            });
        }
    }
    let ok = !windows.is_empty();
    ProviderUsage {
        provider: "claude".into(),
        ok,
        plan: None,
        windows,
        fetched_at: now_ms(),
        error: if ok {
            None
        } else {
            Some("额度接口响应中没有可识别的窗口数据".into())
        },
        model: None,
        source: "official".into(),
        proxy_host: None,
    }
}

fn window_label(key: &str) -> String {
    match key {
        "five_hour" => "5 小时".into(),
        "seven_day" => "本周".into(),
        "seven_day_opus" => "本周 Opus".into(),
        "seven_day_sonnet" => "本周 Sonnet".into(),
        other => other.replace('_', " "),
    }
}

/// 简易 ISO8601 → epoch 秒（只处理 UTC "Z" 结尾的常见格式，失败返回 None）
fn parse_iso_to_epoch(s: &str) -> Option<i64> {
    let s = s.trim();
    let date_part = s.get(0..10)?;
    let time_part = s.get(11..19)?;
    let mut date = date_part.split('-');
    let (y, m, d) = (
        date.next()?.parse::<i64>().ok()?,
        date.next()?.parse::<u32>().ok()?,
        date.next()?.parse::<u32>().ok()?,
    );
    let mut time = time_part.split(':');
    let (hh, mm, ss) = (
        time.next()?.parse::<i64>().ok()?,
        time.next()?.parse::<i64>().ok()?,
        time.next()?.parse::<i64>().ok()?,
    );
    // days_from_civil 算法（Howard Hinnant），将公历日期转为 epoch 天数
    let y_adj = if m <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = (y_adj - era * 400) as i64;
    let mp = ((m + 9) % 12) as i64;
    let doy = (153 * mp + 2) / 5 + (d as i64) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + hh * 3600 + mm * 60 + ss)
}
