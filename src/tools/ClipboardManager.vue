<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore } from "../stores/clipboard";
import type { ClipboardEntry } from "../types";

const store = useClipboardStore();
const input = ref<HTMLInputElement>();
let debounceTimer: ReturnType<typeof setTimeout>;

function onSearchInput() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    store.fetchHistory(store.searchQuery, store.filterType);
  }, 300);
}

function setFilter(type: string | null) {
  store.filterType = type;
  store.fetchHistory(store.searchQuery, type);
}

async function paste(entry: ClipboardEntry) {
  await invoke("paste_entry", { id: entry.id });
}

function getPreview(entry: ClipboardEntry): string {
  if (entry.type === "text" && entry.text_val) {
    return entry.text_val.length > 150
      ? entry.text_val.slice(0, 150) + "..."
      : entry.text_val;
  }
  if (entry.type === "image") return "[图片]";
  if (entry.type === "files" && entry.file_list) {
    const files = JSON.parse(entry.file_list) as string[];
    return files.map((f) => f.split("/").pop() || f).join(", ");
  }
  return "";
}

onMounted(() => store.fetchHistory());
</script>

<template>
  <div class="clipboard-tool">
    <div class="toolbar">
      <div class="search">
        <input
          ref="input"
          v-model="store.searchQuery"
          placeholder="搜索剪贴板..."
          @input="onSearchInput"
        />
      </div>
      <div class="filters">
        <button
          :class="{ active: store.filterType === null }"
          @click="setFilter(null)"
        >
          全部
        </button>
        <button
          :class="{ active: store.filterType === 'text' }"
          @click="setFilter('text')"
        >
          文本
        </button>
        <button
          :class="{ active: store.filterType === 'image' }"
          @click="setFilter('image')"
        >
          图片
        </button>
        <button
          :class="{ active: store.filterType === 'files' }"
          @click="setFilter('files')"
        >
          文件
        </button>
      </div>
    </div>

    <div class="content" v-if="!store.loading">
      <div
        v-for="(entry, idx) in store.entries"
        :key="entry.id"
        class="entry"
        :class="{ selected: idx === store.selectedIndex }"
        @click="store.selectedIndex = idx"
        @dblclick="paste(entry)"
      >
        <div class="entry-icon">
          <span v-if="entry.type === 'text'">📋</span>
          <span v-else-if="entry.type === 'image'">🖼</span>
          <span v-else>📁</span>
        </div>
        <div class="entry-body">
          <div class="entry-preview">{{ getPreview(entry) }}</div>
          <div class="entry-meta">
            <span>{{ entry.source_app || "未知" }}</span>
            <span>·</span>
            <span>{{ new Date(entry.created_at * 1000).toLocaleString() }}</span>
            <span v-if="entry.fav" class="badge">★ 收藏</span>
            <span v-if="entry.pinned" class="badge">📌 置顶</span>
          </div>
        </div>
        <div class="entry-actions">
          <button
            class="action-btn"
            :title="entry.fav ? '取消收藏' : '收藏'"
            @click.stop="store.toggleFavorite(entry.id)"
          >
            {{ entry.fav ? '★' : '☆' }}
          </button>
          <button
            class="action-btn"
            :title="entry.pinned ? '取消置顶' : '置顶'"
            @click.stop="store.togglePin(entry.id)"
          >
            📌
          </button>
          <button
            class="action-btn danger"
            title="删除"
            @click.stop="store.deleteEntry(entry.id)"
          >
            🗑
          </button>
        </div>
      </div>

      <div v-if="store.entries.length === 0" class="empty">
        <p>暂无剪贴板记录</p>
        <p class="hint">复制一些内容后会出现在这里</p>
      </div>
    </div>

    <div v-else class="loading">
      <p>加载中...</p>
    </div>
  </div>
</template>

<style scoped>
.clipboard-tool {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.search {
  flex: 1;
}

.search input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: #1e1e38;
  color: var(--text-bright);
  font-size: 14px;
  outline: none;
}

.search input::placeholder {
  color: var(--text-dim);
}

.filters {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.filters button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: transparent;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.filters button.active {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.content::-webkit-scrollbar {
  width: 6px;
}

.content::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

.entry {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  cursor: pointer;
  transition: background 0.1s;
}

.entry:hover,
.entry.selected {
  background: var(--bg-selected);
}

.entry-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.entry-body {
  flex: 1;
  min-width: 0;
}

.entry-preview {
  font-size: 13px;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.entry-meta {
  display: flex;
  gap: 6px;
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-dim);
  align-items: center;
}

.badge {
  color: var(--accent);
}

.entry-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.entry:hover .entry-actions {
  opacity: 1;
}

.action-btn {
  padding: 4px 6px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-dim);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text);
}

.action-btn.danger:hover {
  background: rgba(233, 69, 96, 0.2);
  color: var(--accent);
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-dim);
  gap: 8px;
}

.empty .hint {
  font-size: 12px;
  opacity: 0.6;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-dim);
}
</style>
