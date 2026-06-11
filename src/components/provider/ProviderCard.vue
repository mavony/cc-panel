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
  if (percent >= 70) return "#f5a623";
  return meta.value.color;
}
</script>

<template>
  <article class="card">
    <header class="card-head">
      <span class="dot" :style="{ backgroundColor: meta.color }" />
      <span class="name">{{ meta.name }}</span>
      <span v-if="usage.source === 'proxy'" class="badge badge-proxy">第三方</span>
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
      <p class="proxy-note">第三方代理，无官方额度数据</p>
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
      <p class="fetched">{{ formatRelative(usage.fetchedAt, now) }}更新</p>
    </template>
    <p v-else class="error">{{ usage.error ?? "额度获取失败" }}</p>
  </article>
</template>

<style scoped>
.card {
  background: #171b22;
  border: 1px solid #262c36;
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
  color: #f5a623;
  border: 1px solid #4a3a1d;
  background: #2a2113;
}

.badge-model {
  color: #9aa7ff;
  border: 1px solid #2b3354;
  background: #181d2e;
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
  color: #8b93a1;
}

.plan {
  font-size: 10px;
  color: #8b93a1;
  border: 1px solid #262c36;
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
  color: #8b93a1;
}

.window-percent {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.bar {
  height: 5px;
  background: #262c36;
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
  color: #8b93a1;
}

.fetched {
  margin: 0;
  font-size: 10px;
  color: #5c6470;
}

.error {
  margin: 0;
  font-size: 11px;
  color: #f5a623;
}
</style>
