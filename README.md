# CC Panel

English | [简体中文](README.zh-CN.md)

> A multi-session monitor for Claude Code / Codex — your agents work across terminals, while one glance at the menu bar tells you who's running, who's waiting for you, and how much quota is left.

A macOS menu bar app built with Tauri 2 + Rust + Vue 3. ~4MB installer, fully local.

![CC Panel](screenshot.png)

## Why

If you run multiple Claude Code / Codex sessions, you know the pain:

- Three or four terminals running agents, and you keep switching between them — only to find one has been stuck on a permission prompt for half an hour
- "Usage limit reached" hits you out of nowhere, with zero warning
- To check what files a session changed or where it is in the task, you scroll through terminal history

## Features

### 📊 Subscription usage at a glance

- Claude 5-hour / weekly window usage with reset times; progress bars change color near the threshold
- Codex usage parsed entirely from local session files — zero extra network requests
- Cards show the model currently in use (e.g. `fable-5[1m]` / `gpt-5.5`)
- Detects third-party proxy configs (`ANTHROPIC_BASE_URL` / non-default `model_provider`) and avoids showing misleading official quota numbers

### 👀 All agent sessions in one screen

- Every active session: title, project, status, and what it's doing right now (e.g. `Edit: claude.rs`)
- Expand for details: task checklist, latest activity, changed files (collapsed to 5 by default, click to reveal in Finder), git branch

### ⏸ Stop waiting on sessions — get notified

- When a session stops at a permission prompt / question / plan approval: an orange "waiting" badge pins it to the top, the menu bar shows a `⏸ n` counter, and a system notification (with sound) fires
- Permission prompts are caught on two paths: instantly via the PreToolUse hook, or by a transcript heuristic (~30s for edits / ~90s for Bash) when the hook isn't active — so a stuck session never goes silent
- Session-finished notifications too (based on ~2 minutes of inactivity)
- Dedup is per prompt (10 minutes), so multiple confirmations in one session each notify; toggles live in Settings

### ✅ Approve without switching terminals

- Via Claude Code's official PreToolUse hook, tool permission prompts appear right in the panel — click Allow / Deny and it takes effect
- Falls back to the native terminal prompt when unhandled (configurable 10–300s, default 45s; hook timeouts are kept in sync automatically); completely transparent when the panel is closed
- Safety gates: pass-through when the panel window is hidden (zero latency), session path validation, special permission modes respected, allowlisted commands skipped — **on any error it only falls back to the terminal, never auto-approves**
- One-click hook install/uninstall in settings (backs up `~/.claude/settings.json` before writing)

### 🗂 Session manager

- Browse and search **all** historical Claude Code / Codex sessions (title / project, provider filter, paginated)
- Read the full conversation of any session, then resume it in your terminal (`claude --resume` / `codex resume`) or open its project folder
- Delete old sessions — two-step confirm, moved to Trash (recoverable); active sessions are protected server-side

### ⚙️ A real settings page

- Theme: dark / light / follow system
- Language: 中文 / English (UI and system notifications)
- Launch at login, notification sound toggle, panel-confirmation timeout, preferred terminal (Terminal / iTerm2)

### 🔒 Security by design

- Fully local: OAuth tokens live in memory only — never written to disk, logs, or the frontend
- No TCP ports; hook communication goes over a unix socket (dir 0700 / socket 0600, current user only)
- The only network request is Anthropic's official usage endpoint (GET, 10s timeout, silent degradation on failure)
- Release builds are Developer ID signed + notarized — no "damaged app" warnings

## Install

Download the DMG from [Releases](https://github.com/mavony/cc-panel/releases) and drag it into Applications (signed & notarized, no `xattr` tricks needed):

- Apple Silicon: `CC Panel_x.y.z_aarch64.dmg`
- Intel: `CC Panel_x.y.z_x64.dmg`

To enable in-panel approval: open the panel → ⚙ (top right) → Settings → check "Confirm tool permissions in panel" → takes effect for newly started Claude Code sessions.

## Development

```bash
pnpm install
pnpm tauri dev               # run in dev mode
cargo run --example probe    # verify data parsing without GUI (run inside src-tauri)
```

For signed + notarized releases: copy `scripts/build-signed.example.sh` to `scripts/build-signed.sh` (gitignored), fill in your Developer ID identity and notarization credentials, then run it — it produces notarized DMGs for both aarch64 and x86_64.

Data sources: `~/.claude/projects/**/*.jsonl` (Claude sessions), `~/.codex/sessions/**/*.jsonl` (Codex sessions), and the Claude OAuth usage endpoint (quota).

## Known limitations

- AskUserQuestion choices are **displayed** in the panel, but answering still happens in the terminal (the CLI reads from its own TTY; there is no injection channel)
- Codex has no equivalent hook mechanism, so approval features are Claude Code only
- Session-finished notifications lag by ~2 minutes (inherent to mtime-quiescence detection)

## License

[MIT](LICENSE)
