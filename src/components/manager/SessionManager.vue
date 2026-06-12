<script setup lang="ts">
import { nextTick, onMounted, shallowRef, watch } from "vue";

import { useNow } from "../../composables/useNow";
import { usePanelStore } from "../../stores/panel";
import type { ChatMessage, HistorySession } from "../../types";
import { formatRelative } from "../../utils/format";

const PAGE_SIZE = 50;

defineEmits<{ back: [] }>();

const store = usePanelStore();
const now = useNow();

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
  return role === "user" ? "我" : role === "assistant" ? "助手" : "工具";
}

onMounted(() => load(true));
</script>

<template>
  <div class="manager">
    <header class="bar">
      <button class="back" title="返回面板" @click="$emit('back')">‹</button>
      <h2 class="bar-title">会话管理</h2>
      <input
        v-model="keyword"
        class="search"
        type="search"
        placeholder="搜索标题或项目…"
      />
      <div class="filter">
        <button
          v-for="opt in [
            { value: '', label: '全部' },
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
      恢复会话失败：{{ store.resumeError }}
      <button class="resume-error-close" @click="store.resumeError = null">✕</button>
    </p>
    <p v-if="store.deleteError" class="resume-error">
      删除会话失败：{{ store.deleteError }}
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
                  <span v-if="s.isActive" class="item-active">进行中</span>
                </span>
              </span>
            </button>
          </li>
        </ul>
        <p v-else-if="!listLoading" class="empty">没有匹配的会话</p>
        <button v-if="hasMore && items.length" class="more" :disabled="listLoading" @click="load(false)">
          {{ listLoading ? "加载中…" : "加载更多" }}
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
              :title="selected.isActive ? '会话正在进行中，原终端仍在运行' : '打开终端并续接此会话'"
              @click="resume"
            >
              {{ resuming ? "正在打开…" : selected.isActive ? "进行中" : "在终端恢复" }}
            </button>
            <button class="reveal-btn" @click="store.revealPath(selected.projectPath)">
              打开目录
            </button>
            <button
              v-if="!selected.isActive"
              class="delete-btn"
              :class="{ 'is-armed': deleteArmed }"
              :title="deleteArmed ? '再次点击确认删除（移到废纸篓，可恢复）' : '删除此会话（移到废纸篓）'"
              @click="removeSession"
            >
              {{ deleteArmed ? "确认删除？" : "删除会话" }}
            </button>
          </div>

          <div ref="messagesEl" class="messages">
            <p v-if="messagesLoading" class="empty">加载对话内容…</p>
            <p v-else-if="!messages.length" class="empty">没有可展示的消息</p>
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
        <p v-else class="empty empty-center">选择左侧会话查看对话内容</p>
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
  background: #171b22;
  border: 1px solid #262c36;
  border-radius: 10px;
  color: #e6e9ee;
  font-size: 18px;
  line-height: 1;
  width: 30px;
  height: 30px;
  cursor: pointer;
}

.back:hover {
  background: #1d222b;
}

.bar-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
}

.search {
  margin-left: auto;
  background: #11141a;
  border: 1px solid #262c36;
  border-radius: 10px;
  color: #e6e9ee;
  font-size: 12px;
  padding: 6px 10px;
  width: 200px;
  outline: none;
}

.search:focus {
  border-color: #3a4250;
}

.filter {
  display: flex;
  background: #11141a;
  border: 1px solid #262c36;
  border-radius: 10px;
  padding: 2px;
  gap: 2px;
}

.filter-btn {
  background: none;
  border: none;
  border-radius: 8px;
  color: #8b93a1;
  font-size: 11px;
  padding: 4px 10px;
  cursor: pointer;
}

.filter-btn.is-on {
  background: #262c36;
  color: #e6e9ee;
}

.resume-error {
  margin: 0;
  font-size: 12px;
  color: #ff8589;
  background: #2a1517;
  border: 1px solid #4a2326;
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
  color: #ff8589;
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
  background: #171b22;
  border: 1px solid #262c36;
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
  background: #1d222b;
}

.item.is-selected {
  background: #1f2733;
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
  color: #8b93a1;
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
  color: #6ea8fe;
  font-size: 11px;
  cursor: pointer;
  padding: 6px;
}

.more:disabled {
  color: #5c6470;
  cursor: default;
}

.right {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: #171b22;
  border: 1px solid #262c36;
  border-radius: 14px;
  overflow: hidden;
}

.detail-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-bottom: 1px solid #262c36;
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
  color: #8b93a1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resume-btn {
  flex-shrink: 0;
  background: #2156c9;
  border: none;
  border-radius: 10px;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  padding: 7px 14px;
  cursor: pointer;
}

.resume-btn:hover:not(:disabled) {
  background: #2a66e8;
}

.resume-btn:disabled {
  background: #262c36;
  color: #8b93a1;
  cursor: default;
}

.reveal-btn {
  flex-shrink: 0;
  background: none;
  border: 1px solid #262c36;
  border-radius: 10px;
  color: #b8bfc9;
  font-size: 12px;
  padding: 6px 12px;
  cursor: pointer;
}

.reveal-btn:hover {
  background: #1d222b;
}

.delete-btn {
  flex-shrink: 0;
  background: none;
  border: 1px solid #262c36;
  border-radius: 10px;
  color: #b8bfc9;
  font-size: 12px;
  padding: 6px 12px;
  cursor: pointer;
}

.delete-btn:hover {
  border-color: #6b2a2a;
  color: #e57373;
}

.delete-btn.is-armed {
  background: #c0392b;
  border-color: #c0392b;
  color: #fff;
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
  color: #8b93a1;
  border: 1px solid #262c36;
  border-radius: 6px;
  padding: 2px 6px;
  margin-top: 1px;
}

.msg-user .msg-role {
  color: #6ea8fe;
  border-color: #2a3a55;
}

.msg-text {
  margin: 0;
  font-size: 12px;
  color: #b8bfc9;
  white-space: pre-wrap;
  word-break: break-word;
  min-width: 0;
}

.msg-user .msg-text {
  color: #e6e9ee;
}

.msg-tool .msg-text {
  color: #5c6470;
  font-size: 11px;
}

.empty {
  margin: 0;
  font-size: 12px;
  color: #5c6470;
  text-align: center;
  padding: 24px 0;
}

.empty-center {
  margin: auto;
}
</style>
