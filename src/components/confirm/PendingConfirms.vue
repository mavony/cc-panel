<script setup lang="ts">
import type { PendingConfirm } from "../../types";

defineProps<{ confirms: PendingConfirm[] }>();

defineEmits<{ resolve: [id: number, decision: "allow" | "deny"] }>();

function projectName(path: string): string {
  const i = path.lastIndexOf("/");
  return i >= 0 ? path.slice(i + 1) : path;
}
</script>

<template>
  <section v-if="confirms.length" class="confirm-section">
    <h2 class="panel-title">待确认（{{ confirms.length }}）</h2>
    <article v-for="c in confirms" :key="c.id" class="confirm-card">
      <div class="confirm-main">
        <span class="tool">{{ c.toolName }}</span>
        <code v-if="c.summary" class="summary" :title="c.summary">{{ c.summary }}</code>
        <span v-if="c.projectPath" class="project">{{ projectName(c.projectPath) }}</span>
      </div>
      <div class="confirm-actions">
        <button class="btn btn-allow" @click="$emit('resolve', c.id, 'allow')">允许</button>
        <button class="btn btn-deny" @click="$emit('resolve', c.id, 'deny')">拒绝</button>
      </div>
      <p class="hint">不操作约 45 秒后回到终端确认</p>
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
  color: #f5a623;
}

.confirm-card {
  background: #201a10;
  border: 1px solid #4a3a1d;
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
  color: #f5a623;
  flex-shrink: 0;
}

.summary {
  font-size: 11px;
  color: #e6e9ee;
  background: #11141a;
  border: 1px solid #262c36;
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
  color: #8b93a1;
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
  background: #1d7a46;
  color: #fff;
}

.btn-allow:hover {
  background: #239454;
}

.btn-deny {
  background: #2a2f3a;
  color: #e6e9ee;
}

.btn-deny:hover {
  background: #3a2528;
  color: #ff8589;
}

.hint {
  margin: 0;
  font-size: 10px;
  color: #8b7355;
}
</style>
