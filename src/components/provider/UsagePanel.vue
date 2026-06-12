<script setup lang="ts">
import type { ProviderUsage } from "../../types";
import ProviderCard from "./ProviderCard.vue";

defineProps<{
  usages: ProviderUsage[];
  loading: boolean;
  now: number;
}>();

defineEmits<{ refresh: [] }>();
</script>

<template>
  <section>
    <div class="panel-head">
      <h2 class="panel-title">{{ $t("usage.title") }}</h2>
      <button
        class="refresh-btn"
        :class="{ 'is-loading': loading }"
        :disabled="loading"
        @click="$emit('refresh')"
      >
        ⟳
      </button>
    </div>
    <div v-if="usages.length" class="cards">
      <ProviderCard
        v-for="usage in usages"
        :key="usage.provider"
        :usage="usage"
        :now="now"
      />
    </div>
    <p v-else-if="loading" class="empty">{{ $t("usage.loading") }}</p>
    <p v-else class="empty">{{ $t("usage.notDetected") }}</p>
  </section>
</template>

<style scoped>
.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.panel-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-dim);
}

.refresh-btn {
  background: none;
  border: none;
  color: var(--text-dim);
  font-size: 15px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 6px;
}

.refresh-btn:hover {
  color: var(--text);
  background: var(--surface-hover);
}

.refresh-btn.is-loading {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 10px;
}

.empty {
  margin: 0;
  font-size: 12px;
  color: var(--text-dim);
}
</style>
