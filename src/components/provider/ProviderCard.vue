<script setup lang="ts">
import { computed } from "vue";

import type { ProviderUsage } from "../../types";
import { formatRelative, formatResetAt } from "../../utils/format";

const props = defineProps<{
  usage: ProviderUsage;
  now: number;
}>();

const meta = computed(() =>
  props.usage.provider === "claude"
    ? { name: "Claude", color: "#d97757" }
    : { name: "Codex", color: "#19c37d" },
);

/** 模型 ID 截短展示：去掉 claude- 前缀和 -YYYYMMDD 日期后缀，完整 ID 放 title */
const shortModel = computed(() => {
  const m = props.usage.model;
  if (!m) return null;
  return m.replace(/^claude-/, "").replace(/-\d{8}$/, "");
});

function barColor(percent: number): string {
  if (percent >= 90) return "#e5484d";
  if (percent >= 70) return "var(--warn)";
  return meta.value.color;
}
</script>

<template>
  <article class="card">
    <header class="card-head">
      <span class="dot" :style="{ backgroundColor: meta.color }" />
      <span class="name">{{ meta.name }}</span>
      <span v-if="usage.source === 'proxy'" class="badge badge-proxy">{{ $t("usage.thirdParty") }}</span>
      <span class="spacer" />
      <span
        v-if="shortModel"
        class="badge badge-model"
        :title="usage.model ?? undefined"
      >{{ shortModel }}</span>
      <span v-if="usage.plan" class="plan">{{ usage.plan }}</span>
    </header>

    <template v-if="usage.source === 'proxy'">
      <p class="proxy-host" :title="usage.proxyHost ?? undefined">{{ usage.proxyHost }}</p>
      <p class="proxy-note">{{ $t("usage.proxyNote") }}</p>
    </template>

    <template v-else-if="usage.ok">
      <div v-for="w in usage.windows" :key="w.label" class="window">
        <div class="window-row">
          <span class="window-label">{{ w.label }}</span>
          <span class="window-percent">{{ Math.round(w.usedPercent) }}%</span>
        </div>
        <div class="bar">
          <div
            class="bar-fill"
            :style="{
              width: `${Math.min(100, w.usedPercent)}%`,
              backgroundColor: barColor(w.usedPercent),
            }"
          />
        </div>
        <p v-if="w.resetsAt" class="window-reset">
          {{ formatResetAt(w.resetsAt, now) }}
        </p>
      </div>
      <p class="fetched">{{ $t("usage.updated", { t: formatRelative(usage.fetchedAt, now) }) }}</p>
    </template>
    <p v-else class="error">{{ usage.error ?? $t("usage.fetchError") }}</p>
  </article>
</template>

<style scoped>
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.card-head {
  display: flex;
  align-items: center;
  gap: 6px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.name {
  font-weight: 600;
  font-size: 13px;
}

.spacer {
  flex: 1;
}

.badge {
  font-size: 10px;
  border-radius: 999px;
  padding: 0 6px;
  white-space: nowrap;
  max-width: 130px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.badge-proxy {
  color: var(--warn);
  border: 1px solid var(--warn-border);
  background: var(--warn-bg);
}

.badge-model {
  color: var(--indigo);
  border: 1px solid var(--accent-border);
  background: var(--accent-bg);
  font-variant-numeric: tabular-nums;
}

.proxy-host {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.proxy-note {
  margin: 0;
  font-size: 11px;
  color: var(--text-dim);
}

.plan {
  font-size: 10px;
  color: var(--text-dim);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 0 6px;
  text-transform: capitalize;
}

.window {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.window-row {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.window-label {
  color: var(--text-dim);
}

.window-percent {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.bar {
  height: 5px;
  background: var(--border);
  border-radius: 3px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.4s ease;
}

.window-reset {
  margin: 0;
  font-size: 10px;
  color: var(--text-dim);
}

.fetched {
  margin: 0;
  font-size: 10px;
  color: var(--text-faint);
}

.error {
  margin: 0;
  font-size: 11px;
  color: var(--warn);
}
</style>
