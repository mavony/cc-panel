//! 会话管理页后端 dry-run：打印历史会话列表与首个会话的消息
//! 运行：cargo run --example history_dry [-- <keyword>]

fn main() {
    let keyword = std::env::args().nth(1);
    let start = std::time::Instant::now();
    let list = cc_panel_lib::debug_history_list(0, 20, keyword.as_deref());
    println!("== 历史会话（前 20，耗时 {:?}）==", start.elapsed());
    for s in &list {
        println!(
            "[{}]{} {} | {} | {}",
            s.provider,
            if s.is_active { "*" } else { " " },
            s.title,
            s.project_path,
            s.file_path
        );
    }

    if let Some(first) = list.first() {
        println!("\n== 首个会话消息（最近 10 条）==");
        match cc_panel_lib::debug_history_messages(&first.file_path, 10) {
            Ok(msgs) => {
                for m in msgs {
                    let text: String = m.text.chars().take(80).collect();
                    println!("[{}] {}", m.role, text.replace('\n', " "));
                }
            }
            Err(e) => println!("ERR: {e}"),
        }
    }
}
