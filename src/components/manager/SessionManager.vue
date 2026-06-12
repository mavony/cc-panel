<script setup lang="ts">
import { nextTick, onMounted, shallowRef, watch } from "vue";

import { useI18n } from "vue-i18n";

import { useNow } from "../../composables/useNow";
import { usePanelStore } from "../../stores/panel";
import type { ChatMessage, HistorySession } from "../../types";
import { formatRelative } from "../../utils/format";

const PAGE_SIZE = 50;

defineEmits<{ back: [] }>();

const store = usePanelStore();
const now = useNow();
const { t } = useI18n();

const items = shallowRef<HistorySession[]>([]);
const listLoading = shallowRef(false);
const hasMore = shallowRef(true);
const keyword = shallowRef("");
const providerFilter = shallowRef<"" | "claude" | "codex">("");

const selected = shallowRef<HistorySession | null>(null);
const messages = shallowRef<ChatMessage[]>([]);
const messagesLoading = shallowRef(false);
const messagesEl = shallowRef<HTMLElement | null>(null);
const resuming = shallowRef(false);

async function load(reset: boolean) {
  if (listLoading.value) return;
  listLoading.value = true;
  const offset = reset ? 0 : items.value.length;
  const batch = await store.fetchHistorySessions(
    offset,
    PAGE_SIZE,
    keyword.value.trim(),
    providerFilter.value,
  );
  items.value = reset ? batch : [...items.value, ...batch];
  hasMore.value = batch.length === PAGE_SIZE;
  listLoading.value = false;
}

let searchTimer: ReturnType<typeof setTimeout> | undefined;
watch([keyword, providerFilter], () => {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => load(true), 300);
});

async function select(session: HistorySession) {
  selected.value = session;
  deleteArmed.value = false;
  messagesLoading.value = true;
  messages.value = await store.fetchSessionMessages(session.filePath);
  messagesLoading.value = false;
  await nextTick();
  messagesEl.value?.scrollTo({ top: messagesEl.value.scrollHeight });
}

async function resume() {
  if (!selected.value || resuming.value) return;
  resuming.value = true;
  await store.resumeSession(selected.value.filePath);
  resuming.value = false;
}

// 删除采用两段式确认：第一次点击进入待确认态，3 秒内再点才执行
const deleteArmed = shallowRef(false);
let deleteArmTimer: ReturnType<typeof setTimeout> | undefined;

async function removeSession() {
  if (!selected.value) return;
  if (!deleteArmed.value) {
    deleteArmed.value = true;
    clearTimeout(deleteArmTimer);
    deleteArmTimer = setTimeout(() => (deleteArmed.value = false), 3000);
    return;
  }
  clearTimeout(deleteArmTimer);
  deleteArmed.value = false;
  const filePath = selected.value.filePath;
  if (await store.deleteSession(filePath)) {
    items.value = items.value.filter((s) => s.filePath !== filePath);
    selected.value = null;
    messages.value = [];
  }
}

function roleLabel(role: ChatMessage["role"]) {
  return role === "user" ? t("manager.roleMe") : role === "assistant" ? t("manager.roleAssistant") : t("manager.roleTool");
}

onMounted(() => load(true));
</script>

<template>
  <div class="manager">
    <header class="bar">
      <button class="back" :title="$t('manager.back')" @click="$emit('back')">‹</button>
      <h2 class="bar-title">{{ $t("manager.title") }}</h2>
      <input
        v-model="keyword"
        class="search"
        type="search"
        :placeholder="$t('manager.searchPlaceholder')"
      />
      <div class="filter">
        <button
          v-for="opt in [
            { value: '', label: t('manager.filterAll') },
            { value: 'claude', label: 'Claude' },
            { value: 'codex', label: 'Codex' },
          ]"
          :key="opt.value"
          class="filter-btn"
          :class="{ 'is-on': providerFilter === opt.value }"
          @click="providerFilter = opt.value as typeof providerFilter"
        >
          {{ opt.label }}
        </button>
      </div>
    </header>

    <p v-if="store.resumeError" class="resume-error">
      {{ $t("manager.resumeFailed") }}{{ store.resumeError }}
      <button class="resume-error-close" @click="store.resumeError = null">✕</button>
    </p>
    <p v-if="store.deleteError" class="resume-error">
      {{ $t("manager.deleteFailed") }}{{ store.deleteError }}
      <button class="resume-error-close" @click="store.deleteError = null">✕</button>
    </p>

    <div class="panes">
      <aside class="left">
        <ul v-if="items.length" class="items">
          <li v-for="s in items" :key="s.filePath">
            <button
              class="item"
              :class="{ 'is-selected': selected?.filePath === s.filePath }"
              @click="select(s)"
            >
              <span
                class="item-dot"
                :style="{ backgroundColor: s.provider === 'claude' ? '#d97757' : '#19c37d' }"
              />
              <span class="item-main">
                <span class="item-title">{{ s.title }}</span>
                <span class="item-meta">
                  {{ s.projectPath.split("/").pop() }} · {{ formatRelative(s.lastActivityAt, now) }}
                  <span v-if="s.isActive" class="item-active">{{ $t("manager.active") }}</span>
                </span>
              </span>
            </button>
          </li>
        </ul>
        <p v-else-if="!listLoading" class="empty">{{ $t("manager.noMatch") }}</p>
        <button v-if="hasMore && items.length" class="more" :disabled="listLoading" @click="load(false)">
          {{ listLoading ? $t("manager.loading") : $t("manager.loadMore") }}
        </button>
      </aside>

      <main class="right">
        <template v-if="selected">
          <div class="detail-bar">
            <div class="detail-info">
              <h3 class="detail-title">{{ selected.title }}</h3>
              <p class="detail-path">{{ selected.projectPath }}</p>
            </div>
            <button
              class="resume-btn"
              :disabled="selected.isActive || resuming"
              :title="selected.isActive ? $t('manager.resumeActiveTitle') : $t('manager.resumeTitle')"
              @click="resume"
            >
              {{ resuming ? $t("manager.resuming") : selected.isActive ? $t("manager.active") : $t("manager.resume") }}
            </button>
            <button class="reveal-btn" @click="store.revealPath(selected.projectPath)">
              {{ $t("manager.openDir") }}
            </button>
            <button
              v-if="!selected.isActive"
              class="delete-btn"
              :class="{ 'is-armed': deleteArmed }"
              :title="deleteArmed ? $t('manager.deleteArmedTitle') : $t('manager.deleteTitle')"
              @click="removeSession"
            >
              {{ deleteArmed ? $t("manager.deleteConfirm") : $t("manager.deleteBtn") }}
            </button>
          </div>

          <div ref="messagesEl" class="messages">
            <p v-if="messagesLoading" class="empty">{{ $t("manager.loadingMessages") }}</p>
            <p v-else-if="!messages.length" class="empty">{{ $t("manager.noMessages") }}</p>
            <template v-else>
              <div
                v-for="(m, i) in messages"
                :key="i"
                class="msg"
                :class="`msg-${m.role}`"
              >
                <span class="msg-role">{{ roleLabel(m.role) }}</span>
                <p class="msg-text">{{ m.text }}</p>
              </div>
            </template>
          </div>
        </template>
        <p v-else class="empty empty-center">{{ $t("manager.selectHint") }}</p>
      </main>
    </div>
  </div>
</template>

<style scoped>
.manager {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.bar {
  display: flex;
  align-items: center;
  gap: 10px;
}

.back {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  color: var(--text);
  font-size: 18px;
  line-height: 1;
  width: 30px;
  height: 30px;
  cursor: pointer;
}

.back:hover {
  background: var(--surface-hover);
}

.bar-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
}

.search {
  margin-left: auto;
  background: var(--surface-deep);
  border: 1px solid var(--border);
  border-radius: 10px;
  color: var(--text);
  font-size: 12px;
  padding: 6px 10px;
  width: 200px;
  outline: none;
}

.search:focus {
  border-color: var(--border-strong);
}

.filter {
  display: flex;
  background: var(--surface-deep);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 2px;
  gap: 2px;
}

.filter-btn {
  background: none;
  border: none;
  border-radius: 8px;
  color: var(--text-dim);
  font-size: 11px;
  padding: 4px 10px;
  cursor: pointer;
}

.filter-btn.is-on {
  background: var(--border);
  color: var(--text);
}

.resume-error {
  margin: 0;
  font-size: 12px;
  color: var(--danger-text);
  background: var(--danger-bg);
  border: 1px solid var(--danger-border);
  border-radius: 10px;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.resume-error-close {
  background: none;
  border: none;
  color: var(--danger-text);
  font-size: 11px;
  cursor: pointer;
  padding: 0 2px;
}

.panes {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.left {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 8px;
}

.items {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  width: 100%;
  background: none;
  border: none;
  border-radius: 10px;
  color: inherit;
  font: inherit;
  text-align: left;
  padding: 8px;
  cursor: pointer;
}

.item:hover {
  background: var(--surface-hover);
}

.item.is-selected {
  background: var(--surface-raised);
}

.item-dot {
  flex-shrink: 0;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  margin-top: 5px;
}

.item-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-title {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-meta {
  font-size: 11px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-active {
  color: #19c37d;
  margin-left: 4px;
}

.more {
  background: none;
  border: none;
  color: var(--accent-text);
  font-size: 11px;
  cursor: pointer;
  padding: 6px;
}

.more:disabled {
  color: var(--text-faint);
  cursor: default;
}

.right {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}

.detail-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-bottom: 1px solid var(--border);
}

.detail-info {
  flex: 1;
  min-width: 0;
}

.detail-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.detail-path {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resume-btn {
  flex-shrink: 0;
  background: var(--accent);
  border: none;
  border-radius: 10px;
  color: var(--text-on-accent);
  font-size: 12px;
  font-weight: 600;
  padding: 7px 14px;
  cursor: pointer;
}

.resume-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.resume-btn:disabled {
  background: var(--border);
  color: var(--text-dim);
  cursor: default;
}

.reveal-btn {
  flex-shrink: 0;
  background: none;
  border: 1px solid var(--border);
  border-radius: 10px;
  color: var(--text-mid);
  font-size: 12px;
  padding: 6px 12px;
  cursor: pointer;
}

.reveal-btn:hover {
  background: var(--surface-hover);
}

.delete-btn {
  flex-shrink: 0;
  background: none;
  border: 1px solid var(--border);
  border-radius: 10px;
  color: var(--text-mid);
  font-size: 12px;
  padding: 6px 12px;
  cursor: pointer;
}

.delete-btn:hover {
  border-color: var(--danger-border);
  color: var(--danger-muted);
}

.delete-btn.is-armed {
  background: var(--danger-strong);
  border-color: var(--danger-strong);
  color: var(--text-on-accent);
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.msg {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.msg-role {
  flex-shrink: 0;
  font-size: 10px;
  color: var(--text-dim);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 6px;
  margin-top: 1px;
}

.msg-user .msg-role {
  color: var(--accent-text);
  border-color: var(--accent-border);
}

.msg-text {
  margin: 0;
  font-size: 12px;
  color: var(--text-mid);
  white-space: pre-wrap;
  word-break: break-word;
  min-width: 0;
}

.msg-user .msg-text {
  color: var(--text);
}

.msg-tool .msg-text {
  color: var(--text-faint);
  font-size: 11px;
}

.empty {
  margin: 0;
  font-size: 12px;
  color: var(--text-faint);
  text-align: center;
  padding: 24px 0;
}

.empty-center {
  margin: auto;
}
</style>
