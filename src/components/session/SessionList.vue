<script setup lang="ts">
import { shallowRef } from "vue";

import type { SessionDetail } from "../../types";
import SessionCard from "./SessionCard.vue";

defineProps<{
  sessions: SessionDetail[];
  now: number;
}>();

defineEmits<{ reveal: [path: string] }>();

const expandedPath = shallowRef<string | null>(null);

function toggle(path: string) {
  expandedPath.value = expandedPath.value === path ? null : path;
}
</script>

<template>
  <section class="list-section">
    <h2 class="panel-title">Agent 会话</h2>
    <div v-if="sessions.length" class="list">
      <SessionCard
        v-for="session in sessions"
        :key="session.filePath"
        :session="session"
        :now="now"
        :expanded="expandedPath === session.filePath"
        @toggle="toggle(session.filePath)"
        @reveal="$emit('reveal', $event)"
      />
    </div>
    <p v-else class="empty">最近 30 分钟内没有活跃会话</p>
  </section>
</template>

<style scoped>
.list-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.panel-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: #8b93a1;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty {
  margin: 0;
  font-size: 12px;
  color: #5c6470;
  text-align: center;
  padding: 24px 0;
}
</style>
