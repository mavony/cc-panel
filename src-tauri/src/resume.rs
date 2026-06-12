//! 在终端中恢复会话：claude --resume <id> / codex resume <id>

use std::io::BufRead;
use std::path::{Path, PathBuf};

/// 36 位标准 UUID（8-4-4-4-12，十六进制）
fn is_uuid(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    s.char_indices().all(|(i, c)| match i {
        8 | 13 | 18 | 23 => c == '-',
        _ => c.is_ascii_hexdigit(),
    })
}

/// 校验 file_path 是已知数据源下的会话文件，返回 (provider, session_id)
pub(crate) fn parse_target(file_path: &str) -> Result<(&'static str, String), String> {
    let path = Path::new(file_path)
        .canonicalize()
        .map_err(|_| crate::tr_rt("会话文件不存在", "Session file not found"))?;
    if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
        return Err(crate::tr_rt("不是会话文件", "Not a session file").into());
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or(crate::tr_rt("无法解析文件名", "Cannot parse file name"))?;

    let under = |root: Option<PathBuf>| {
        root.and_then(|r| r.canonicalize().ok())
            .is_some_and(|r| path.starts_with(r))
    };

    if under(crate::sources::claude::projects_dir()) {
        // Claude：文件名即 sessionId
        if is_uuid(stem) {
            return Ok(("claude", stem.to_string()));
        }
    } else if under(crate::sources::codex::sessions_dir()) {
        // Codex：rollout-<时间戳>-<uuid>.jsonl，取末尾 UUID
        if stem.len() > 36 {
            let tail = &stem[stem.len() - 36..];
            if is_uuid(tail) {
                return Ok(("codex", tail.to_string()));
            }
        }
    }
    Err(crate::tr_rt("不在已知会话目录中", "Not in a known session directory").into())
}

/// 从会话文件头部找项目目录（Claude 顶层 cwd / Codex session_meta payload.cwd）
fn session_cwd(path: &str) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines().take(100).flatten() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let cwd = v
            .get("cwd")
            .or_else(|| v.get("payload").and_then(|p| p.get("cwd")))
            .and_then(serde_json::Value::as_str);
        if let Some(c) = cwd {
            return Some(c.to_string());
        }
    }
    None
}

/// 单引号包裹的 shell 转义（内嵌单引号替换为 '\''）
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// AppleScript 字符串字面量转义
fn applescript_quote(s: &str) -> String {
    s.replace('\\', r"\\").replace('"', r#"\""#)
}

/// 校验会话文件并组装恢复用的 shell 命令
pub fn build_shell_cmd(file_path: &str) -> Result<String, String> {
    let (provider, session_id) = parse_target(file_path)?;

    let dir = session_cwd(file_path).ok_or(crate::tr_rt("会话文件中没有项目目录信息", "No project directory recorded in the session"))?;
    if !Path::new(&dir).is_dir() {
        return Err(format!("{}{dir}", crate::tr_rt("项目目录已不存在：", "Project directory no longer exists: ")));
    }

    let resume_cmd = match provider {
        "claude" => format!("claude --resume {session_id}"),
        _ => format!("codex resume {session_id}"),
    };
    Ok(format!("cd {} && {}", shell_quote(&dir), resume_cmd))
}

/// 在用户偏好的终端中执行恢复命令
pub fn resume_in_terminal(file_path: &str) -> Result<(), String> {
    let shell_cmd = build_shell_cmd(file_path)?;
    let escaped = applescript_quote(&shell_cmd);

    let script = match crate::load_panel_settings().terminal_app.as_str() {
        "iTerm" => format!(
            r#"tell application "iTerm"
    activate
    create window with default profile
    tell current session of current window
        write text "{escaped}"
    end tell
end tell"#
        ),
        _ => format!(
            r#"tell application "Terminal"
    activate
    do script "{escaped}"
end tell"#
        ),
    };

    let out = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("{}{e}", crate::tr_rt("无法调用 osascript: ", "Failed to invoke osascript: ")))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!("{}{}", crate::tr_rt("打开终端失败：", "Failed to open terminal: "), err.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_check() {
        assert!(is_uuid("019d4869-0b53-7392-ac68-650a86d04e98"));
        assert!(!is_uuid("019d4869-0b53-7392-ac68-650a86d04e9"));
        assert!(!is_uuid("019d4869_0b53_7392_ac68_650a86d04e98"));
        assert!(!is_uuid("../../etc/passwd; rm -rf ~ aaaaaaaaaaa"));
    }

    #[test]
    fn quoting() {
        assert_eq!(shell_quote("/a b/c"), "'/a b/c'");
        assert_eq!(shell_quote("a'b"), r"'a'\''b'");
        assert_eq!(applescript_quote(r#"say "hi" \ bye"#), r#"say \"hi\" \\ bye"#);
    }
}
