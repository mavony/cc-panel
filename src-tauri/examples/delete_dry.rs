// 临时 dry-run：只验证 delete 的校验路径，不真正删除任何会话
fn main() {
    // 1) 目录外文件 → 应拒绝
    std::fs::write("/tmp/fake.jsonl", "{}").unwrap();
    println!("目录外: {:?}", cc_panel_lib::debug_delete_session("/tmp/fake.jsonl"));
    // 2) 进行中的会话（找一个 mtime < 120s 的真实文件）→ 应拒绝
    let active = std::process::Command::new("sh")
        .args(["-c", "find ~/.claude/projects -name '*.jsonl' -mtime -2m 2>/dev/null | head -1"])
        .output().unwrap();
    let p = String::from_utf8_lossy(&active.stdout).trim().to_string();
    if !p.is_empty() {
        println!("进行中: {:?}", cc_panel_lib::debug_delete_session(&p));
    } else {
        println!("进行中: (无 mtime<2m 的会话可测)");
    }
}
