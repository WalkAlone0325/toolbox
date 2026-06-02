<script setup lang="ts">
import { onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore } from "../stores/clipboard";
import type { ClipboardEntry, EntryType } from "../types";

const store = useClipboardStore();
let debounceTimer: ReturnType<typeof setTimeout>;

function onSearchInput() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    store.fetchHistory(store.searchQuery, store.filterType);
  }, 250);
}

function setFilter(type: EntryType | null) {
  store.filterType = type;
  store.fetchHistory(store.searchQuery, type);
}

async function paste(entry: ClipboardEntry) {
  await invoke("paste_entry", { id: entry.id });
}

async function copyAndPaste(entry: ClipboardEntry) {
  await paste(entry);
}

function getPreview(entry: ClipboardEntry): string {
  if (entry.type === "text" && entry.text_val) {
    return entry.text_val.length > 200
      ? entry.text_val.slice(0, 200) + "..."
      : entry.text_val;
  }
  if (entry.type === "image") return "图片内容";
  if (entry.type === "files" && entry.file_list) {
    try {
      const files = JSON.parse(entry.file_list) as string[];
      return files.map((f) => f.split("/").pop() || f).join("  ·  ");
    } catch {
      return "文件列表";
    }
  }
  return "";
}

function formatSize(bytes: number): string {
  if (!bytes) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function formatTime(ts: number): string {
  const now = Date.now();
  const diff = Math.floor((now - ts * 1000) / 1000);
  if (diff < 5) return "刚刚";
  if (diff < 60) return `${diff} 秒前`;
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 86400 * 7) return `${Math.floor(diff / 86400)} 天前`;
  const d = new Date(ts * 1000);
  const y = d.getFullYear();
  const now2 = new Date();
  const sameYear = y === now2.getFullYear();
  return d.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: sameYear ? "2-digit" : undefined,
    minute: sameYear ? "2-digit" : undefined,
    year: sameYear ? undefined : "numeric",
  });
}

const filterTabs = computed<{ id: EntryType | null; label: string }[]>(() => [
  { id: null, label: "全部" },
  { id: "text", label: "文本" },
  { id: "image", label: "图片" },
  { id: "files", label: "文件" },
]);

const typeMeta: Record<EntryType, { label: string; color: string; bg: string }> = {
  text: { label: "T", color: "#60a5fa", bg: "rgba(59, 130, 246, 0.12)" },
  image: { label: "IMG", color: "#34d399", bg: "rgba(16, 185, 129, 0.12)" },
  files: { label: "F", color: "#fbbf24", bg: "rgba(245, 158, 11, 0.12)" },
};

onMounted(() => store.fetchHistory());
</script>

<template>
  <div class="clipboard-tool">
    <div class="toolbar">
      <div class="search">
        <svg
          class="search-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="11" cy="11" r="7" />
          <path d="m20 20-3.5-3.5" stroke-linecap="round" />
        </svg>
        <input
          v-model="store.searchQuery"
          placeholder="搜索剪贴板内容..."
          class="search-input"
          @input="onSearchInput"
        />
        <kbd v-if="!store.searchQuery" class="search-kbd">⌘ K</kbd>
      </div>

      <div class="filters">
        <button
          v-for="tab in filterTabs"
          :key="tab.id ?? 'all'"
          class="filter-tab"
          :class="{ active: store.filterType === tab.id }"
          @click="setFilter(tab.id)"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <div class="list-wrap">
      <div v-if="store.loading" class="state loading">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <div v-else-if="store.entries.length === 0" class="state empty">
        <svg viewBox="0 0 64 64" fill="none" class="empty-icon">
          <rect x="14" y="18" width="36" height="32" rx="6" stroke="currentColor" stroke-width="2.5" />
          <rect x="24" y="12" width="16" height="8" rx="2" stroke="currentColor" stroke-width="2.5" />
          <path d="M24 34h16M24 42h10" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
        </svg>
        <p class="empty-title">暂无剪贴板记录</p>
        <p class="empty-hint">复制一些内容后会自动出现在这里</p>
      </div>

      <div v-else class="entries">
        <article
          v-for="(entry, idx) in store.entries"
          :key="entry.id"
          class="entry"
          :class="{
            selected: idx === store.selectedIndex,
            pinned: entry.pinned,
          }"
          @click="store.selectedIndex = idx"
          @dblclick="copyAndPaste(entry)"
        >
          <div
            class="entry-type-badge"
            :style="{
              color: typeMeta[entry.type].color,
              background: typeMeta[entry.type].bg,
            }"
          >
            {{ typeMeta[entry.type].label }}
          </div>

          <div class="entry-body">
            <div class="entry-preview">{{ getPreview(entry) }}</div>
            <div class="entry-meta">
              <span class="meta-item time">{{ formatTime(entry.created_at) }}</span>
              <span v-if="entry.source_app" class="meta-sep">·</span>
              <span v-if="entry.source_app" class="meta-item">{{
                entry.source_app
              }}</span>
              <span v-if="entry.byte_size" class="meta-sep">·</span>
              <span v-if="entry.byte_size" class="meta-item">{{
                formatSize(entry.byte_size)
              }}</span>
              <span v-if="entry.fav" class="meta-fav">★ 收藏</span>
            </div>
          </div>

          <div class="entry-actions">
            <button
              class="action-btn"
              :class="{ active: entry.fav }"
              :title="entry.fav ? '取消收藏' : '收藏'"
              @click.stop="store.toggleFavorite(entry.id)"
            >
              <svg v-if="entry.fav" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 17.27 18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 17.27 18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" stroke-linejoin="round" />
              </svg>
            </button>
            <button
              class="action-btn"
              :class="{ active: entry.pinned }"
              :title="entry.pinned ? '取消置顶' : '置顶'"
              @click.stop="store.togglePin(entry.id)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2v8M12 10l-4 4h8l-4-4zM12 14v6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
            <button
              class="action-btn primary"
              title="复制并粘贴"
              @click.stop="copyAndPaste(entry)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="8" y="8" width="12" height="12" rx="2" />
                <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
              </svg>
            </button>
            <button
              class="action-btn danger"
              title="删除"
              @click.stop="store.deleteEntry(entry.id)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M6 6l1 14a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-14M10 11v6M14 11v6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<style scoped>
.clipboard-tool {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  background: rgba(17, 20, 27, 0.4);
}

.search {
  position: relative;
  flex: 1;
  max-width: 480px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 15px;
  height: 15px;
  color: var(--text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 8px 56px 8px 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  color: var(--text-bright);
  font-size: 13px;
  outline: none;
  transition: all var(--transition);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-input:focus {
  border-color: var(--border-focus);
  background: var(--bg-elev-3);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.12);
}

.search-kbd {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  padding: 2px 6px;
  border: 1px solid var(--border-strong);
  border-radius: 4px;
  background: var(--bg-elev-1);
  color: var(--text-muted);
  font-size: 10.5px;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  pointer-events: none;
}

.filters {
  display: flex;
  gap: 2px;
  padding: 3px;
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  flex-shrink: 0;
}

.filter-tab {
  padding: 5px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}

.filter-tab:hover {
  color: var(--text);
}

.filter-tab.active {
  background: var(--accent);
  color: white;
  box-shadow: 0 2px 6px var(--accent-glow);
}

.list-wrap {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  gap: 10px;
  padding: 40px 20px;
}

.state.empty {
  gap: 16px;
}

.empty-icon {
  width: 64px;
  height: 64px;
  color: var(--text-muted);
  opacity: 0.4;
}

.empty-title {
  font-size: 14px;
  color: var(--text-dim);
  font-weight: 500;
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.loading {
  flex-direction: row;
  gap: 12px;
  font-size: 13px;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border-strong);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.entries {
  padding: 8px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.entry {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px 10px 10px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: background var(--transition);
  border: 1px solid transparent;
}

.entry:hover {
  background: var(--bg-elev-1);
  border-color: var(--border);
}

.entry.selected {
  background: var(--bg-active);
  border-color: rgba(99, 102, 241, 0.3);
}

.entry.pinned::before {
  content: "";
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 60%;
  background: var(--warning);
  border-radius: 0 2px 2px 0;
}

.entry-type-badge {
  width: 38px;
  height: 38px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  flex-shrink: 0;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
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
  line-height: 1.4;
}

.entry.selected .entry-preview {
  color: var(--text-bright);
}

.entry-meta {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-top: 3px;
  font-size: 11px;
  color: var(--text-muted);
  flex-wrap: nowrap;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.meta-item.time {
  color: var(--text-dim);
  font-variant-numeric: tabular-nums;
}

.meta-sep {
  color: var(--text-muted);
  opacity: 0.5;
}

.meta-fav {
  color: var(--warning);
  margin-left: 4px;
}

.entry-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transform: translateX(4px);
  transition: all var(--transition);
  flex-shrink: 0;
}

.entry:hover .entry-actions,
.entry.selected .entry-actions {
  opacity: 1;
  transform: translateX(0);
}

.action-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  transition: all var(--transition);
}

.action-btn svg {
  width: 15px;
  height: 15px;
}

.action-btn:hover {
  background: var(--bg-elev-3);
  color: var(--text);
}

.action-btn.active {
  color: var(--warning);
}

.action-btn.primary {
  color: var(--accent);
}

.action-btn.primary:hover {
  background: rgba(99, 102, 241, 0.15);
}

.action-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: var(--danger);
}
</style>
