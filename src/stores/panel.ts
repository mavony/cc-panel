import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, shallowRef } from "vue";

import type {
  ChatMessage,
  HistorySession,
  PanelSettings,
  PendingConfirm,
  ProviderUsage,
  SessionDetail,
} from "../types";

const SESSIONS_POLL_MS = 5000;
const USAGE_POLL_MS = 60000;
const CONFIRM_POLL_MS = 1500;

export const usePanelStore = defineStore("panel", () => {
  const sessions = shallowRef<SessionDetail[]>([]);
  const usages = shallowRef<ProviderUsage[]>([]);
  const usageLoading = shallowRef(false);
  const lastSyncAt = shallowRef<number | null>(null);
  const pendingConfirms = shallowRef<PendingConfirm[]>([]);
  const confirmHookEnabled = shallowRef(false);
  const panelSettings = shallowRef<PanelSettings>({
    notifyConfirm: true,
    notifyDone: true,
    terminalApp: "Terminal",
  });
  const resumeError = shallowRef<string | null>(null);

  const runningCount = computed(
    () => sessions.value.filter((s) => s.status === "running").length,
  );
  const waitingCount = computed(
    () => sessions.value.filter((s) => s.status === "waiting").length,
  );

  async function refreshSessions() {
    try {
      sessions.value = await invoke<SessionDetail[]>("list_sessions");
      lastSyncAt.value = Date.now();
    } catch {
      // 单次轮询失败保留上次数据
    }
  }

  async function refreshUsage() {
    usageLoading.value = true;
    try {
      usages.value = await invoke<ProviderUsage[]>("get_usage");
    } catch {
      // 保留上次数据
    } finally {
      usageLoading.value = false;
    }
  }

  async function refreshConfirms() {
    try {
      pendingConfirms.value = await invoke<PendingConfirm[]>("list_pending_confirms");
    } catch {
      // 保留上次数据
    }
  }

  /** 允许/拒绝某个待确认请求；后端返回 false 表示已超时回落终端 */
  async function resolveConfirm(id: number, decision: "allow" | "deny") {
    try {
      await invoke<boolean>("resolve_confirm", { id, decision });
    } finally {
      pendingConfirms.value = pendingConfirms.value.filter((c) => c.id !== id);
    }
  }

  async function loadPanelSettings() {
    try {
      panelSettings.value = await invoke<PanelSettings>("get_panel_settings");
    } catch {
      // 保留默认值
    }
  }

  async function updatePanelSettings(patch: Partial<PanelSettings>) {
    const next = { ...panelSettings.value, ...patch };
    panelSettings.value = next;
    try {
      await invoke("set_panel_settings", { settings: next });
    } catch {
      // 写入失败保留 UI 状态，下次启动回读
    }
  }

  async function loadConfirmHookStatus() {
    try {
      confirmHookEnabled.value = await invoke<boolean>("confirm_hook_status");
    } catch {
      confirmHookEnabled.value = false;
    }
  }

  /** 安装/卸载面板内确认 hook，返回错误文案（成功为 null） */
  async function setConfirmHook(enabled: boolean): Promise<string | null> {
    try {
      confirmHookEnabled.value = await invoke<boolean>("set_confirm_hook", { enabled });
      return null;
    } catch (e) {
      return String(e);
    }
  }

  function startPolling() {
    refreshSessions();
    refreshUsage();
    refreshConfirms();
    loadConfirmHookStatus();
    loadPanelSettings();
    setInterval(refreshSessions, SESSIONS_POLL_MS);
    setInterval(refreshUsage, USAGE_POLL_MS);
    setInterval(refreshConfirms, CONFIRM_POLL_MS);
  }

  async function revealPath(path: string) {
    try {
      await invoke("reveal_path", { path });
    } catch {
      // 路径可能已被删除，静默忽略
    }
  }

  /** 在终端中恢复会话，失败时把原因放进 resumeError 供 UI 展示 */
  async function resumeSession(filePath: string) {
    resumeError.value = null;
    try {
      await invoke("resume_session", { filePath });
    } catch (e) {
      resumeError.value = String(e);
    }
  }

  /** 会话管理页：分页拉取历史会话 */
  async function fetchHistorySessions(
    offset: number,
    limit: number,
    keyword: string,
    provider: "" | "claude" | "codex",
  ): Promise<HistorySession[]> {
    try {
      return await invoke<HistorySession[]>("list_history_sessions", {
        offset,
        limit,
        keyword: keyword || null,
        provider: provider || null,
      });
    } catch {
      return [];
    }
  }

  /** 会话管理页：拉取单个会话的对话消息 */
  async function fetchSessionMessages(filePath: string, max = 200): Promise<ChatMessage[]> {
    try {
      return await invoke<ChatMessage[]>("get_session_messages", { filePath, max });
    } catch {
      return [];
    }
  }

  return {
    sessions,
    usages,
    usageLoading,
    lastSyncAt,
    pendingConfirms,
    confirmHookEnabled,
    panelSettings,
    updatePanelSettings,
    runningCount,
    waitingCount,
    refreshSessions,
    refreshUsage,
    resolveConfirm,
    setConfirmHook,
    startPolling,
    revealPath,
    resumeError,
    resumeSession,
    fetchHistorySessions,
    fetchSessionMessages,
  };
});
