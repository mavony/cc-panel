//! 命令行探针：不启动 GUI，直接验证两个数据源的扫描/解析结果
//! 运行：cargo run --example probe

fn main() {
    println!("== Claude 会话 ==");
    for s in cc_panel_lib::debug_scan_claude() {
        print_session(&s);
    }
    println!("\n== Codex 会话 ==");
    for s in cc_panel_lib::debug_scan_codex() {
        print_session(&s);
    }
    println!("\n== Codex 额度（本地 rate_limits）==");
    println!(
        "{}",
        serde_json::to_string_pretty(&cc_panel_lib::debug_codex_usage()).unwrap()
    );
}

fn print_session(s: &serde_json::Value) {
    println!(
        "[{}] {} | {} | {} 条消息 | todos:{} outputs:{} | {}",
        s["status"].as_str().unwrap_or("?"),
        s["title"].as_str().unwrap_or("?"),
        s["projectPath"].as_str().unwrap_or("?"),
        s["messageCount"],
        s["todos"].as_array().map(|a| a.len()).unwrap_or(0),
        s["outputs"].as_array().map(|a| a.len()).unwrap_or(0),
        s["currentActivity"].as_str().unwrap_or(""),
    );
}
