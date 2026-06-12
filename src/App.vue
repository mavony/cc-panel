<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, shallowRef } from "vue";

import type { PanelSettings } from "./types";

import PendingConfirms from "./components/confirm/PendingConfirms.vue";
import SessionManager from "./components/manager/SessionManager.vue";
import UsagePanel from "./components/provider/UsagePanel.vue";
import SessionList from "./components/session/SessionList.vue";
import { useNow } from "./composables/useNow";
import { usePanelStore } from "./stores/panel";
import { formatRelative } from "./utils/format";

const store = usePanelStore();
const now = useNow();

/** 点通知/回到面板时由后端推送：聚焦展开对应会话 */
const focusReq = shallowRef<{ path: string; ts: number } | null>(null);

/** 当前视图：主面板 / 会话管理页 */
const view = shallowRef<"panel" | "sessions">("panel");

onMounted(() => {
  store.startPolling();
  listen<string>("focus-session", (e) => {
    view.value = "panel"; // 聚焦联动只在主面板有意义
    focusReq.value = { path: e.payload, ts: Date.now() };
  });
});

const settingsOpen = shallowRef(false);
const hookError = shallowRef<string | null>(null);

async function toggleConfirmHook(e: Event) {
  const enabled = (e.target as HTMLInputElement).checked;
  hookError.value = await store.setConfirmHook(enabled);
}

/** data-tauri-drag-region 属性探测在 Overlay 模式下不可靠，显式调用窗口 API 拖拽 */
function startDrag(e: MouseEvent) {
  if (e.buttons !== 1) return;
  const win = getCurrentWindow();
  if (e.detail === 2) {
    win.toggleMaximize();
  } else {
    win.startDragging();
  }
}
</script>

<template>
  <div class="app" :class="{ 'is-manager': view === 'sessions' }">
    <!-- 红绿灯所在高度的全宽拖拽条（原生标题栏已隐藏） -->
    <div class="drag-strip" @mousedown="startDrag" />

    <SessionManager v-if="view === 'sessions'" @back="view = 'panel'" />

    <template v-else>
    <header class="header" @mousedown="startDrag">
      <div>
        <h1 class="header-title">CC Panel</h1>
        <p class="header-sub">
          {{ store.runningCount > 0 ? `${store.runningCount} 个 agent 进行中` : "暂无进行中的 agent" }}
        </p>
      </div>
      <span class="header-right" @mousedown.stop>
        <span v-if="store.lastSyncAt" class="header-sync">
          {{ formatRelative(store.lastSyncAt, now) }}同步
        </span>
        <button class="gear" title="会话管理" @click="view = 'sessions'">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="16" rx="3" />
            <path d="M7 9h10M7 13h10M7 17h6" />
          </svg>
        </button>
        <button class="gear" :class="{ 'is-open': settingsOpen }" title="设置" @click="settingsOpen = !settingsOpen">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </span>
    </header>

    <section v-if="settingsOpen" class="settings">
      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.confirmHookEnabled"
          @change="toggleConfirmHook"
        />
        <span>
          在面板中确认工具权限（Claude）
          <span class="settings-note">
            安装 PreToolUse hook 到 ~/.claude/settings.json（写入前自动备份）；面板窗口可见时，Bash/文件写入的权限确认会出现在面板里，新开的会话生效
          </span>
        </span>
      </label>
      <p v-if="hookError" class="settings-error">{{ hookError }}</p>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.panelSettings.notifyConfirm"
          @change="store.updatePanelSettings({ notifyConfirm: ($event.target as HTMLInputElement).checked })"
        />
        <span>
          待确认时发系统通知
          <span class="settings-note">会话停在权限/选择/计划批准上时提醒（同一会话 10 分钟内不重复）</span>
        </span>
      </label>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.panelSettings.notifyDone"
          @change="store.updatePanelSettings({ notifyDone: ($event.target as HTMLInputElement).checked })"
        />
        <span>
          会话结束时发系统通知
          <span class="settings-note">基于约 2 分钟无新输出判定，通知有相应延迟</span>
        </span>
      </label>

      <label class="settings-row settings-row-select">
        <span>
          恢复会话使用的终端
          <span class="settings-note">点击会话里的"在终端恢复"时，用所选终端打开并续接会话</span>
        </span>
        <select
          class="settings-select"
          :value="store.panelSettings.terminalApp"
          @change="store.updatePanelSettings({ terminalApp: ($event.target as HTMLSelectElement).value as PanelSettings['terminalApp'] })"
        >
          <option value="Terminal">Terminal</option>
          <option value="iTerm">iTerm2</option>
        </select>
      </label>
    </section>

    <div class="columns">
      <aside class="col-usage">
        <UsagePanel
          :usages="store.usages"
          :loading="store.usageLoading"
          :now="now"
          @refresh="store.refreshUsage"
        />
      </aside>

      <main class="col-sessions">
        <PendingConfirms
          :confirms="store.pendingConfirms"
          @resolve="store.resolveConfirm"
        />
        <SessionList
          :sessions="store.sessions"
          :now="now"
          :focus="focusReq"
          @reveal="store.revealPath"
        />
      </main>
    </div>
    </template>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 0 18px 24px;
  min-height: 100vh;
  box-sizing: border-box;
}

/* 会话管理页：固定视口高度，列表/消息流在各自栏内滚动 */
.app.is-manager {
  height: 100vh;
  overflow: hidden;
  padding-bottom: 18px;
}

.drag-strip {
  height: 30px;
  margin: 0 -18px;
  flex-shrink: 0;
}

.columns {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  flex: 1;
}

.col-usage {
  width: 240px;
  flex-shrink: 0;
  position: sticky;
  top: 18px;
}

.col-sessions {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.header-right {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.gear {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: #8b93a1;
  font-size: 15px;
  cursor: pointer;
  padding: 0 4px;
  border-radius: 6px;
  line-height: 1.4;
  height: 21px;
}

.gear:hover,
.gear.is-open {
  color: #e6e9ee;
  background: #1d222b;
}

.settings {
  background: #171b22;
  border: 1px solid #262c36;
  border-radius: 14px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.settings-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 12px;
  cursor: pointer;
}

.settings-row input {
  margin-top: 2px;
}

.settings-note {
  display: block;
  margin-top: 2px;
  font-size: 11px;
  color: #8b93a1;
}

.settings-error {
  margin: 8px 0 0;
  font-size: 11px;
  color: #ff8589;
}

.settings-row-select {
  justify-content: space-between;
  align-items: center;
  cursor: default;
}

.settings-select {
  background: #11141a;
  border: 1px solid #262c36;
  border-radius: 8px;
  color: #e6e9ee;
  font-size: 12px;
  padding: 4px 8px;
}

/* 窄窗口退化为上下结构 */
@media (max-width: 759px) {
  .columns {
    flex-direction: column;
    align-items: stretch;
  }

  .col-usage {
    width: auto;
    position: static;
  }
}

.header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.header-title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.header-sub {
  margin: 2px 0 0;
  font-size: 12px;
  color: #8b93a1;
}

.header-sync {
  font-size: 11px;
  color: #8b93a1;
  padding-top: 4px;
}
</style>
