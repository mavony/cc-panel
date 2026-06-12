<script setup lang="ts">
import { usePanelStore } from "../../stores/panel";
import type { PendingConfirm } from "../../types";

const store = usePanelStore();

defineProps<{ confirms: PendingConfirm[] }>();

defineEmits<{ resolve: [id: number, decision: "allow" | "deny"] }>();

function projectName(path: string): string {
  const i = path.lastIndexOf("/");
  return i >= 0 ? path.slice(i + 1) : path;
}
</script>

<template>
  <section v-if="confirms.length" class="confirm-section">
    <h2 class="panel-title">{{ $t("confirm.title", { n: confirms.length }) }}</h2>
    <article v-for="c in confirms" :key="c.id" class="confirm-card">
      <div class="confirm-main">
        <span class="tool">{{ c.toolName }}</span>
        <code v-if="c.summary" class="summary" :title="c.summary">{{ c.summary }}</code>
        <span v-if="c.projectPath" class="project">{{ projectName(c.projectPath) }}</span>
      </div>
      <div class="confirm-actions">
        <button class="btn btn-allow" @click="$emit('resolve', c.id, 'allow')">{{ $t("confirm.allow") }}</button>
        <button class="btn btn-deny" @click="$emit('resolve', c.id, 'deny')">{{ $t("confirm.deny") }}</button>
      </div>
      <p class="hint">{{ $t("confirm.hint", { n: store.panelSettings.confirmTimeoutSecs }) }}</p>
    </article>
  </section>
</template>

<style scoped>
.confirm-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.panel-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--warn);
}

.confirm-card {
  background: var(--warn-bg);
  border: 1px solid var(--warn-border);
  border-radius: 14px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.confirm-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.tool {
  font-size: 12px;
  font-weight: 700;
  color: var(--warn);
  flex-shrink: 0;
}

.summary {
  font-size: 11px;
  color: var(--text);
  background: var(--surface-deep);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex: 1;
}

.project {
  font-size: 10px;
  color: var(--text-dim);
  flex-shrink: 0;
}

.confirm-actions {
  display: flex;
  gap: 8px;
}

.btn {
  border: none;
  border-radius: 8px;
  padding: 5px 16px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.btn-allow {
  background: var(--success-strong);
  color: var(--text-on-accent);
}

.btn-allow:hover {
  background: var(--success-hover);
}

.btn-deny {
  background: var(--surface-neutral);
  color: var(--text);
}

.btn-deny:hover {
  background: var(--danger-bg-raised);
  color: var(--danger-text);
}

.hint {
  margin: 0;
  font-size: 10px;
  color: var(--warn-dim);
}
</style>
