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

/** 面板自有设置（~/.cc_panel/settings.json） */
export interface PanelSettings {
  notifyConfirm: boolean;
  notifyDone: boolean;
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
