<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, shallowRef } from "vue";

import PendingConfirms from "./components/confirm/PendingConfirms.vue";
import UsagePanel from "./components/provider/UsagePanel.vue";
import SessionList from "./components/session/SessionList.vue";
import { useNow } from "./composables/useNow";
import { usePanelStore } from "./stores/panel";
import { formatRelative } from "./utils/format";

const store = usePanelStore();
const now = useNow();

onMounted(() => store.startPolling());

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
  <div class="app">
    <!-- 红绿灯所在高度的全宽拖拽条（原生标题栏已隐藏） -->
    <div class="drag-strip" @mousedown="startDrag" />

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
        <button class="gear" :class="{ 'is-open': settingsOpen }" @click="settingsOpen = !settingsOpen">⚙</button>
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
          @reveal="store.revealPath"
        />
      </main>
    </div>
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
  background: none;
  border: none;
  color: #8b93a1;
  font-size: 15px;
  cursor: pointer;
  padding: 0 4px;
  border-radius: 6px;
  line-height: 1.4;
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
