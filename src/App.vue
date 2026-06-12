<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, shallowRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";

import PendingConfirms from "./components/confirm/PendingConfirms.vue";
import SessionManager from "./components/manager/SessionManager.vue";
import UsagePanel from "./components/provider/UsagePanel.vue";
import SessionList from "./components/session/SessionList.vue";
import SettingsPage from "./components/settings/SettingsPage.vue";
import { useNow } from "./composables/useNow";
import { usePanelStore } from "./stores/panel";
import { formatRelative } from "./utils/format";

const store = usePanelStore();
const now = useNow();

/** 点通知/回到面板时由后端推送：聚焦展开对应会话 */
const focusReq = shallowRef<{ path: string; ts: number } | null>(null);

/** 当前视图：主面板 / 会话管理页 / 设置页 */
const view = shallowRef<"panel" | "sessions" | "settings">("panel");

onMounted(() => {
  store.startPolling();
  listen<string>("focus-session", (e) => {
    view.value = "panel"; // 聚焦联动只在主面板有意义
    focusReq.value = { path: e.payload, ts: Date.now() };
  });
});

// 主题：dark/light 直接生效；system 跟随 prefers-color-scheme 并监听变化
const systemDark = window.matchMedia("(prefers-color-scheme: dark)");
const systemDarkNow = shallowRef(systemDark.matches);
systemDark.addEventListener("change", (e) => (systemDarkNow.value = e.matches));
watchEffect(() => {
  const t = store.panelSettings.theme;
  const resolved = t === "system" ? (systemDarkNow.value ? "dark" : "light") : t;
  document.documentElement.dataset.theme = resolved;
});

// 语言：跟随设置切换 i18n locale
const { locale } = useI18n();
watchEffect(() => {
  locale.value = store.panelSettings.language;
});

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
    <SettingsPage v-else-if="view === 'settings'" @back="view = 'panel'" />

    <template v-else>
    <header class="header" @mousedown="startDrag">
      <div>
        <h1 class="header-title">CC Panel</h1>
        <p class="header-sub">
          {{ store.runningCount > 0 ? $t("header.running", { n: store.runningCount }) : $t("header.idle") }}
        </p>
      </div>
      <span class="header-right" @mousedown.stop>
        <span v-if="store.lastSyncAt" class="header-sync">
          {{ $t("header.sync", { t: formatRelative(store.lastSyncAt, now) }) }}
        </span>
        <button class="gear" :title="$t('header.manageTitle')" @click="view = 'sessions'">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="16" rx="3" />
            <path d="M7 9h10M7 13h10M7 17h6" />
          </svg>
        </button>
        <button class="gear" :title="$t('header.settingsTitle')" @click="view = 'settings'">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </span>
    </header>

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
  color: var(--text-dim);
  font-size: 15px;
  cursor: pointer;
  padding: 0 4px;
  border-radius: 6px;
  line-height: 1.4;
  height: 21px;
}

.gear:hover {
  color: var(--text);
  background: var(--surface-hover);
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
  color: var(--text-dim);
}

.header-sync {
  font-size: 11px;
  color: var(--text-dim);
  padding-top: 4px;
}
</style>
