<script setup lang="ts">
import { computed } from "vue";

import type { SessionDetail } from "../../types";
import { formatRelative } from "../../utils/format";
import SessionDetailView from "./SessionDetail.vue";

const props = defineProps<{
  session: SessionDetail;
  now: number;
  expanded: boolean;
}>();

defineEmits<{ toggle: []; reveal: [path: string] }>();

const providerColor = computed(() =>
  props.session.provider === "claude" ? "#d97757" : "#19c37d",
);

const projectName = computed(() => {
  const p = props.session.projectPath;
  const i = p.lastIndexOf("/");
  return i >= 0 ? p.slice(i + 1) : p;
});

const isRunning = computed(() => props.session.status === "running");
const isWaiting = computed(() => props.session.status === "waiting");
</script>

<template>
  <article class="session" :class="{ 'is-expanded': expanded }">
    <button class="session-head" @click="$emit('toggle')">
      <span
        class="status-dot"
        :class="{ 'is-running': isRunning || isWaiting }"
        :style="{
          backgroundColor: isWaiting ? 'var(--warn)' : isRunning ? providerColor : 'var(--text-faint)',
        }"
      />
      <span class="session-main">
        <span class="session-title">{{ session.title }}</span>
        <span v-if="isWaiting" class="waiting-line">
          ⏸ {{ $t("sessions.waiting") }}<template v-if="session.pendingQuestion">: {{ session.pendingQuestion }}</template>
        </span>
        <span class="session-meta">
          <span class="provider" :style="{ color: providerColor }">
            {{ session.provider === "claude" ? "Claude" : "Codex" }}
          </span>
          <template v-if="projectName"> · {{ projectName }}</template>
          · {{ formatRelative(session.lastActivityAt, now) }}
        </span>
      </span>
      <span class="session-side">
        <span v-if="isWaiting" class="waiting-badge">{{ $t("sessions.waitingBadge") }}</span>
        <span v-else-if="isRunning && session.currentActivity" class="activity">
          {{ session.currentActivity }}
        </span>
        <span class="chevron" :class="{ 'is-open': expanded }">›</span>
      </span>
    </button>

    <SessionDetailView
      v-if="expanded"
      :session="session"
      @reveal="$emit('reveal', $event)"
    />
  </article>
</template>

<style scoped>
.session {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}

.session.is-expanded {
  border-color: var(--border-strong);
}

.session-head {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 12px;
  background: none;
  border: none;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.session-head:hover {
  background: var(--surface-hover);
}

.status-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.is-running {
  animation: pulse 1.6s ease-in-out infinite;
}

@keyframes pulse {
  50% {
    opacity: 0.35;
  }
}

.session-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.session-title {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-meta {
  font-size: 11px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.provider {
  font-weight: 600;
}

.session-side {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 40%;
}

.waiting-line {
  font-size: 11px;
  color: var(--warn);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.waiting-badge {
  font-size: 10px;
  color: var(--warn);
  background: var(--warn-bg);
  border: 1px solid var(--warn-border);
  border-radius: 999px;
  padding: 2px 8px;
  white-space: nowrap;
}

.activity {
  font-size: 10px;
  color: var(--text-dim);
  background: var(--surface-deep);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 2px 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chevron {
  color: var(--text-faint);
  font-size: 16px;
  transition: transform 0.2s ease;
}

.chevron.is-open {
  transform: rotate(90deg);
}
</style>
