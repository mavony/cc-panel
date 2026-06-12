<script setup lang="ts">
import { nextTick, shallowRef, watch } from "vue";

import type { SessionDetail } from "../../types";
import SessionCard from "./SessionCard.vue";

const props = defineProps<{
  sessions: SessionDetail[];
  now: number;
  /** 外部请求聚焦的会话（点通知联动），ts 变化即触发 */
  focus?: { path: string; ts: number } | null;
}>();

defineEmits<{ reveal: [path: string] }>();

const expandedPath = shallowRef<string | null>(null);

function toggle(path: string) {
  expandedPath.value = expandedPath.value === path ? null : path;
}

watch(
  () => props.focus,
  async (f) => {
    if (!f) return;
    expandedPath.value = f.path;
    await nextTick();
    const el = document.querySelector(`[data-session="${CSS.escape(f.path)}"]`);
    if (!el) return;
    el.scrollIntoView({ behavior: "smooth", block: "start" });
    el.classList.remove("flash");
    void (el as HTMLElement).offsetWidth; // 重启动画
    el.classList.add("flash");
    setTimeout(() => el.classList.remove("flash"), 2200);
  },
);
</script>

<template>
  <section class="list-section">
    <h2 class="panel-title">Agent 会话</h2>
    <div v-if="sessions.length" class="list">
      <div
        v-for="session in sessions"
        :key="session.filePath"
        :data-session="session.filePath"
        class="item"
      >
        <SessionCard
          :session="session"
          :now="now"
          :expanded="expandedPath === session.filePath"
          @toggle="toggle(session.filePath)"
          @reveal="$emit('reveal', $event)"
        />
      </div>
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

.item {
  border-radius: 14px;
}

.item.flash {
  animation: focus-flash 2.2s ease-out;
}

@keyframes focus-flash {
  0%,
  40% {
    box-shadow: 0 0 0 2px #f5a623;
  }
  100% {
    box-shadow: 0 0 0 2px transparent;
  }
}

.empty {
  margin: 0;
  font-size: 12px;
  color: #5c6470;
  text-align: center;
  padding: 24px 0;
}
</style>
