<script setup lang="ts">
import { computed, shallowRef } from "vue";

import type { SessionDetail } from "../../types";
import { baseName } from "../../utils/format";
import TodoChecklist from "./TodoChecklist.vue";

const props = defineProps<{ session: SessionDetail }>();

defineEmits<{ reveal: [path: string] }>();

const doneCount = computed(
  () => props.session.todos.filter((t) => t.status === "completed").length,
);

/** 输出文件超过该数量时折叠 */
const OUTPUTS_PREVIEW = 5;
const outputsExpanded = shallowRef(false);

const visibleOutputs = computed(() =>
  outputsExpanded.value
    ? props.session.outputs
    : props.session.outputs.slice(0, OUTPUTS_PREVIEW),
);
const hiddenOutputCount = computed(
  () => props.session.outputs.length - visibleOutputs.value.length,
);
</script>

<template>
  <div class="detail">
    <section v-if="session.todos.length" class="block">
      <h3 class="block-title">
        进度
        <span class="block-count">{{ doneCount }}/{{ session.todos.length }}</span>
      </h3>
      <TodoChecklist :todos="session.todos" />
    </section>

    <section v-if="session.latestMessage" class="block">
      <h3 class="block-title">最新动态</h3>
      <p class="message">{{ session.latestMessage }}</p>
    </section>

    <section v-if="session.outputs.length" class="block">
      <h3 class="block-title">输出 <span class="block-count">{{ session.outputs.length }}</span></h3>
      <ul class="outputs">
        <li v-for="path in visibleOutputs" :key="path">
          <button class="output-item" :title="path" @click="$emit('reveal', path)">
            <span class="output-icon">📄</span>
            <span class="output-name">{{ baseName(path) }}</span>
          </button>
        </li>
      </ul>
      <button
        v-if="hiddenOutputCount > 0 || outputsExpanded"
        class="outputs-toggle"
        @click="outputsExpanded = !outputsExpanded"
      >
        {{ outputsExpanded ? "收起" : `展开全部（还有 ${hiddenOutputCount} 个）` }}
      </button>
    </section>

    <footer class="foot">
      <span class="foot-meta">{{ session.messageCount }} 条消息</span>
      <span v-if="session.gitBranch" class="foot-meta">{{ session.gitBranch }}</span>
      <button
        v-if="session.projectPath"
        class="foot-link"
        @click="$emit('reveal', session.projectPath)"
      >
        打开项目目录
      </button>
    </footer>
  </div>
</template>

<style scoped>
.detail {
  border-top: 1px solid #262c36;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.block-title {
  margin: 0;
  font-size: 11px;
  font-weight: 600;
  color: #8b93a1;
  display: flex;
  align-items: center;
  gap: 6px;
}

.block-count {
  font-weight: 400;
  color: #5c6470;
}

.message {
  margin: 0;
  font-size: 12px;
  color: #b8bfc9;
  background: #11141a;
  border-radius: 8px;
  padding: 8px 10px;
  white-space: pre-wrap;
  word-break: break-word;
}

.outputs {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.output-item {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  background: none;
  border: none;
  color: #b8bfc9;
  font: inherit;
  font-size: 12px;
  padding: 4px 6px;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
}

.output-item:hover {
  background: #1d222b;
  color: #e6e9ee;
}

.output-icon {
  font-size: 11px;
}

.output-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.outputs-toggle {
  align-self: flex-start;
  background: none;
  border: none;
  color: #6ea8fe;
  font-size: 11px;
  cursor: pointer;
  padding: 2px 6px;
}

.outputs-toggle:hover {
  text-decoration: underline;
}

.foot {
  display: flex;
  align-items: center;
  gap: 10px;
  border-top: 1px solid #1d222b;
  padding-top: 10px;
}

.foot-meta {
  font-size: 11px;
  color: #5c6470;
}

.foot-link {
  margin-left: auto;
  background: none;
  border: none;
  color: #6ea8fe;
  font-size: 11px;
  cursor: pointer;
  padding: 0;
}

.foot-link:hover {
  text-decoration: underline;
}
</style>
