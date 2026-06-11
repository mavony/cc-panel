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
          backgroundColor: isWaiting ? '#f5a623' : isRunning ? providerColor : '#5c6470',
        }"
      />
      <span class="session-main">
        <span class="session-title">{{ session.title }}</span>
        <span v-if="isWaiting" class="waiting-line">
          ⏸ 等待确认<template v-if="session.pendingQuestion">：{{ session.pendingQuestion }}</template>
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
        <span v-if="isWaiting" class="waiting-badge">待确认</span>
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
  background: #171b22;
  border: 1px solid #262c36;
  border-radius: 14px;
  overflow: hidden;
}

.session.is-expanded {
  border-color: #323a47;
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
  background: #1d222b;
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
  color: #8b93a1;
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
  color: #f5a623;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.waiting-badge {
  font-size: 10px;
  color: #f5a623;
  background: #2a2113;
  border: 1px solid #4a3a1d;
  border-radius: 999px;
  padding: 2px 8px;
  white-space: nowrap;
}

.activity {
  font-size: 10px;
  color: #8b93a1;
  background: #11141a;
  border: 1px solid #262c36;
  border-radius: 999px;
  padding: 2px 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chevron {
  color: #5c6470;
  font-size: 16px;
  transition: transform 0.2s ease;
}

.chevron.is-open {
  transform: rotate(90deg);
}
</style>
