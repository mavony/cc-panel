use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    /// jsonl 文件绝对路径，同时作为会话的稳定标识
    pub file_path: String,
    pub provider: String,
    pub title: String,
    pub project_path: String,
    pub status: String,
    pub started_at: Option<String>,
    pub last_activity_at: i64,
    pub message_count: u32,
    pub current_activity: String,
    /// 会话正在等待用户确认时的问题摘要（按界面语言本地化，仅展示用）
    pub pending_question: Option<String>,
    /// 同一停顿的语言无关标识，用作通知去重 key（不随界面语言变化）
    pub pending_key: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub content: String,
    pub status: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    #[serde(flatten)]
    pub summary: SessionSummary,
    pub todos: Vec<TodoItem>,
    pub outputs: Vec<String>,
    pub latest_message: String,
    pub git_branch: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    /// epoch 秒
    pub resets_at: Option<i64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub provider: String,
    pub ok: bool,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    /// epoch 毫秒
    pub fetched_at: i64,
    pub error: Option<String>,
    /// 当前使用的模型 ID（来自 CLI 配置，缺省时取最新会话）
    pub model: Option<String>,
    /// "official" | "proxy"
    pub source: String,
    /// 第三方代理时仅保留 host，不含路径与参数
    pub proxy_host: Option<String>,
}

/// 文件 mtime 距今不足该秒数视为"进行中"
pub const RUNNING_SECS: u64 = 120;
/// 距今不足该秒数视为"最近活动"，超过则不在列表展示
pub const RECENT_SECS: u64 = 1800;

pub fn status_for_age(age_secs: u64) -> Option<&'static str> {
    if age_secs < RUNNING_SECS {
        Some("running")
    } else if age_secs < RECENT_SECS {
        Some("recent")
    } else {
        None
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn truncate_chars(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max {
        t.to_string()
    } else {
        let cut: String = t.chars().take(max).collect();
        format!("{cut}…")
    }
}
