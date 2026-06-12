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
        {{ $t("sessions.progress") }}
        <span class="block-count">{{ doneCount }}/{{ session.todos.length }}</span>
      </h3>
      <TodoChecklist :todos="session.todos" />
    </section>

    <section v-if="session.latestMessage" class="block">
      <h3 class="block-title">{{ $t("sessions.latest") }}</h3>
      <p class="message">{{ session.latestMessage }}</p>
    </section>

    <section v-if="session.outputs.length" class="block">
      <h3 class="block-title">{{ $t("sessions.outputs") }} <span class="block-count">{{ session.outputs.length }}</span></h3>
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
        {{ outputsExpanded ? $t("sessions.collapse") : $t("sessions.expandAll", { n: hiddenOutputCount }) }}
      </button>
    </section>

    <footer class="foot">
      <span class="foot-meta">{{ $t("sessions.msgCount", { n: session.messageCount }) }}</span>
      <span v-if="session.gitBranch" class="foot-meta">{{ session.gitBranch }}</span>
      <button
        v-if="session.projectPath"
        class="foot-link"
        @click="$emit('reveal', session.projectPath)"
      >
        {{ $t("sessions.openProject") }}
      </button>
    </footer>
  </div>
</template>

<style scoped>
.detail {
  border-top: 1px solid var(--border);
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
  color: var(--text-dim);
  display: flex;
  align-items: center;
  gap: 6px;
}

.block-count {
  font-weight: 400;
  color: var(--text-faint);
}

.message {
  margin: 0;
  font-size: 12px;
  color: var(--text-mid);
  background: var(--surface-deep);
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
  color: var(--text-mid);
  font: inherit;
  font-size: 12px;
  padding: 4px 6px;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
}

.output-item:hover {
  background: var(--surface-hover);
  color: var(--text);
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
  color: var(--accent-text);
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
  border-top: 1px solid var(--surface-hover);
  padding-top: 10px;
}

.foot-meta {
  font-size: 11px;
  color: var(--text-faint);
}

.foot-link {
  margin-left: auto;
  background: none;
  border: none;
  color: var(--accent-text);
  font-size: 11px;
  cursor: pointer;
  padding: 0;
}

.foot-link:hover {
  text-decoration: underline;
}
</style>
