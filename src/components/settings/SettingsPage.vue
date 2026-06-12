<script setup lang="ts">
import { shallowRef } from "vue";

import { usePanelStore } from "../../stores/panel";
import type { PanelSettings } from "../../types";

defineEmits<{ back: [] }>();

const store = usePanelStore();
const hookError = shallowRef<string | null>(null);

async function toggleConfirmHook(e: Event) {
  const enabled = (e.target as HTMLInputElement).checked;
  hookError.value = await store.setConfirmHook(enabled);
}

/** 失焦/回车时 clamp 到 10–300 再保存（后端还会兜底一次） */
function saveConfirmTimeout(e: Event) {
  const input = e.target as HTMLInputElement;
  const v = Math.min(300, Math.max(10, Math.round(Number(input.value) || 45)));
  input.value = String(v);
  if (v !== store.panelSettings.confirmTimeoutSecs) {
    store.updatePanelSettings({ confirmTimeoutSecs: v });
  }
}
</script>

<template>
  <div class="settings-page">
    <header class="bar">
      <button class="back" title="返回面板" @click="$emit('back')">‹</button>
      <h2 class="bar-title">设置</h2>
    </header>

    <section class="settings">
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

      <label class="settings-row settings-row-select">
        <span>
          面板确认等待时长（秒）
          <span class="settings-note">面板内确认超过该时长未处理时，回落到终端原生提示；范围 10–300，默认 45。修改后自动更新已安装的 hook</span>
        </span>
        <input
          class="settings-number"
          type="number"
          min="10"
          max="300"
          step="5"
          :value="store.panelSettings.confirmTimeoutSecs"
          @change="saveConfirmTimeout"
        />
      </label>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.panelSettings.notifyConfirm"
          @change="store.updatePanelSettings({ notifyConfirm: ($event.target as HTMLInputElement).checked })"
        />
        <span>
          待确认时发系统通知
          <span class="settings-note">会话停在权限/选择/计划批准上时提醒（同一确认点 10 分钟内不重复）</span>
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
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.bar {
  display: flex;
  align-items: center;
  gap: 10px;
}

.back {
  background: #171b22;
  border: 1px solid #262c36;
  border-radius: 10px;
  color: #e6e9ee;
  font-size: 18px;
  line-height: 1;
  width: 30px;
  height: 30px;
  cursor: pointer;
}

.back:hover {
  background: #1d222b;
}

.bar-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
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

.settings-row input[type="checkbox"] {
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
  gap: 16px;
}

.settings-select {
  background: #11141a;
  border: 1px solid #262c36;
  border-radius: 8px;
  color: #e6e9ee;
  font-size: 12px;
  padding: 4px 8px;
}

.settings-number {
  background: #11141a;
  border: 1px solid #262c36;
  border-radius: 8px;
  color: #e6e9ee;
  font-size: 12px;
  padding: 4px 8px;
  width: 72px;
  flex-shrink: 0;
}
</style>
