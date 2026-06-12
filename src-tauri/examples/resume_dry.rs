//! 恢复会话 dry-run：只打印将要执行的 shell 命令，不真正打开终端
//! 运行：cargo run --example resume_dry -- <jsonl 路径>

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("用法：cargo run --example resume_dry -- <jsonl 路径>");
        std::process::exit(1);
    };
    match cc_panel_lib::debug_resume_cmd(&path) {
        Ok(cmd) => println!("OK: {cmd}"),
        Err(e) => println!("ERR: {e}"),
    }
}
