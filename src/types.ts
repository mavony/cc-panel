export type Provider = "claude" | "codex";

export type SessionStatus = "waiting" | "running" | "recent";

export type TodoStatus = "pending" | "in_progress" | "completed" | string;

export interface TodoItem {
  content: string;
  status: TodoStatus;
}

export interface SessionDetail {
  /** jsonl 文件绝对路径，同时作为会话的稳定标识 */
  filePath: string;
  provider: Provider;
  title: string;
  projectPath: string;
  status: SessionStatus;
  startedAt: string | null;
  lastActivityAt: number;
  messageCount: number;
  currentActivity: string;
  /** 等待确认时的问题摘要 */
  pendingQuestion: string | null;
  todos: TodoItem[];
  outputs: string[];
  latestMessage: string;
  gitBranch: string | null;
}

/** 会话管理页的历史会话条目 */
export interface HistorySession {
  /** jsonl 文件绝对路径，同时作为会话的稳定标识 */
  filePath: string;
  provider: Provider;
  title: string;
  projectPath: string;
  /** epoch 毫秒 */
  lastActivityAt: number;
  /** 正在进行中（恢复按钮置灰） */
  isActive: boolean;
}

/** 会话管理页的对话消息 */
export interface ChatMessage {
  role: "user" | "assistant" | "tool";
  text: string;
  /** ISO 时间戳 */
  at: string | null;
}

/** 面板自有设置（~/.cc_panel/settings.json） */
export interface PanelSettings {
  notifyConfirm: boolean;
  notifyDone: boolean;
  /** 恢复会话使用的终端 App */
  terminalApp: "Terminal" | "iTerm";
  /** 面板内确认的等待时长（秒），超时回落终端；后端 clamp 10–300 */
  confirmTimeoutSecs: number;
}

/** 来自 PreToolUse hook 的待确认工具权限请求 */
export interface PendingConfirm {
  id: number;
  toolName: string;
  summary: string;
  projectPath: string;
  transcriptPath: string;
  /** epoch 毫秒 */
  createdAt: number;
}

export interface UsageWindow {
  label: string;
  usedPercent: number;
  /** epoch 秒 */
  resetsAt: number | null;
}

export interface ProviderUsage {
  provider: Provider;
  ok: boolean;
  plan: string | null;
  windows: UsageWindow[];
  /** epoch 毫秒 */
  fetchedAt: number;
  error: string | null;
  /** 当前使用的模型 ID */
  model: string | null;
  source: "official" | "proxy";
  /** 第三方代理 host（仅域名） */
  proxyHost: string | null;
}
