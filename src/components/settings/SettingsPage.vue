<script setup lang="ts">
import { shallowRef } from "vue";

import { usePanelStore } from "../../stores/panel";
import type { PanelSettings } from "../../types";

defineEmits<{ back: [] }>();

const store = usePanelStore();
const hookError = shallowRef<string | null>(null);
const autostartError = shallowRef<string | null>(null);

async function toggleConfirmHook(e: Event) {
  const enabled = (e.target as HTMLInputElement).checked;
  hookError.value = await store.setConfirmHook(enabled);
}

async function toggleAutostart(e: Event) {
  const enabled = (e.target as HTMLInputElement).checked;
  autostartError.value = await store.setAutostart(enabled);
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
      <button class="back" :title="$t('settings.back')" @click="$emit('back')">‹</button>
      <h2 class="bar-title">{{ $t("settings.title") }}</h2>
    </header>

    <section class="settings">
      <label class="settings-row settings-row-select">
        <span>
          {{ $t("settings.theme") }}
          <span class="settings-note">{{ $t("settings.themeNote") }}</span>
        </span>
        <select
          class="settings-select"
          :value="store.panelSettings.theme"
          @change="store.updatePanelSettings({ theme: ($event.target as HTMLSelectElement).value as PanelSettings['theme'] })"
        >
          <option value="dark">{{ $t("settings.themeDark") }}</option>
          <option value="light">{{ $t("settings.themeLight") }}</option>
          <option value="system">{{ $t("settings.themeSystem") }}</option>
        </select>
      </label>

      <label class="settings-row settings-row-select">
        <span>
          {{ $t("settings.language") }}
          <span class="settings-note">{{ $t("settings.languageNote") }}</span>
        </span>
        <select
          class="settings-select"
          :value="store.panelSettings.language"
          @change="store.updatePanelSettings({ language: ($event.target as HTMLSelectElement).value as PanelSettings['language'] })"
        >
          <option value="zh">中文</option>
          <option value="en">English</option>
        </select>
      </label>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.autostartEnabled"
          @change="toggleAutostart"
        />
        <span>
          {{ $t("settings.autostart") }}
          <span class="settings-note">{{ $t("settings.autostartNote") }}</span>
        </span>
      </label>
      <p v-if="autostartError" class="settings-error">{{ autostartError }}</p>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.confirmHookEnabled"
          @change="toggleConfirmHook"
        />
        <span>
          {{ $t("settings.hook") }}
          <span class="settings-note">{{ $t("settings.hookNote") }}</span>
        </span>
      </label>
      <p v-if="hookError" class="settings-error">{{ hookError }}</p>

      <label class="settings-row settings-row-select">
        <span>
          {{ $t("settings.timeout") }}
          <span class="settings-note">{{ $t("settings.timeoutNote") }}</span>
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
          {{ $t("settings.notifyConfirm") }}
          <span class="settings-note">{{ $t("settings.notifyConfirmNote") }}</span>
        </span>
      </label>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.panelSettings.notifyDone"
          @change="store.updatePanelSettings({ notifyDone: ($event.target as HTMLInputElement).checked })"
        />
        <span>
          {{ $t("settings.notifyDone") }}
          <span class="settings-note">{{ $t("settings.notifyDoneNote") }}</span>
        </span>
      </label>

      <label class="settings-row">
        <input
          type="checkbox"
          :checked="store.panelSettings.notifySound"
          @change="store.updatePanelSettings({ notifySound: ($event.target as HTMLInputElement).checked })"
        />
        <span>
          {{ $t("settings.notifySound") }}
          <span class="settings-note">{{ $t("settings.notifySoundNote") }}</span>
        </span>
      </label>

      <label class="settings-row settings-row-select">
        <span>
          {{ $t("settings.terminal") }}
          <span class="settings-note">{{ $t("settings.terminalNote") }}</span>
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
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  color: var(--text);
  font-size: 18px;
  line-height: 1;
  width: 30px;
  height: 30px;
  cursor: pointer;
}

.back:hover {
  background: var(--surface-hover);
}

.bar-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
}

.settings {
  background: var(--surface);
  border: 1px solid var(--border);
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
  color: var(--text-dim);
}

.settings-error {
  margin: 8px 0 0;
  font-size: 11px;
  color: var(--danger-text);
}

.settings-row-select {
  justify-content: space-between;
  align-items: center;
  cursor: default;
  gap: 16px;
}

.settings-select {
  background: var(--surface-deep);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 12px;
  padding: 4px 8px;
}

.settings-number {
  background: var(--surface-deep);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 12px;
  padding: 4px 8px;
  width: 72px;
  flex-shrink: 0;
}
</style>
