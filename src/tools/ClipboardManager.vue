<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useClipboardStore } from "../stores/clipboard";
import { useThemeStore } from "../stores/theme";
import { useToastStore } from "../stores/toast";
import type { ClipboardEntry, EntryType } from "../types";

const store = useClipboardStore();
const theme = useThemeStore();
const toast = useToastStore();
const searchInputRef = ref<HTMLInputElement>();
const previewEntry = ref<ClipboardEntry | null>(null);
const showClearConfirm = ref(false);
let unlistenUpdate: UnlistenFn | undefined;

function setFilter(type: EntryType | null) {
  store.filterType = type;
  store.selectedIndex = 0;
}

function focusSearch() {
  searchInputRef.value?.focus();
  searchInputRef.value?.select();
}

function clearSearch() {
  if (store.searchQuery) {
    store.searchQuery = "";
  } else {
    searchInputRef.value?.blur();
  }
}

function onGlobalKeydown(e: KeyboardEvent) {
  const isModK = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k";
  if (isModK) {
    e.preventDefault();
    focusSearch();
    return;
  }
  if (e.key === "Escape") {
    if (showClearConfirm.value) {
      e.preventDefault();
      showClearConfirm.value = false;
      return;
    }
    if (previewEntry.value) {
      e.preventDefault();
      previewEntry.value = null;
      return;
    }
    const active = document.activeElement;
    if (active === searchInputRef.value) {
      e.preventDefault();
      clearSearch();
    }
  }
}

function selectIndex(idx: number) {
  if (idx >= 0 && idx < store.entries.length) {
    store.selectedIndex = idx;
  }
}

async function onToggleFavorite(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  try {
    await store.toggleFavorite(entry.id);
    toast.success(entry.fav ? "已取消收藏" : "已加入收藏");
  } catch (e) {
    toast.danger("操作失败");
  }
}

async function onTogglePin(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  try {
    await store.togglePin(entry.id);
    toast.success(entry.pinned ? "已取消置顶" : "已置顶");
  } catch (e) {
    toast.danger("操作失败");
  }
}

async function onDelete(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  try {
    await store.deleteEntry(entry.id);
    toast.warning("已删除");
  } catch (e) {
    toast.danger("删除失败");
  }
}

function askClearAll() {
  if (store.allEntries.length === 0) {
    toast.info("暂无可清空的记录");
    return;
  }
  showClearConfirm.value = true;
}

async function confirmClearAll() {
  try {
    await store.clearAll();
    showClearConfirm.value = false;
    toast.success("已清空全部记录");
  } catch (e) {
    toast.danger("清空失败");
  }
}

function cancelClearAll() {
  showClearConfirm.value = false;
}

async function onPaste(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  try {
    await invoke("paste_entry", { id: entry.id });
    toast.success("已复制到剪贴板");
  } catch (e) {
    toast.danger("复制失败");
  }
}

async function onEntryDoubleClick(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  await invoke("paste_entry", { id: entry.id });
}

function openPreview(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  previewEntry.value = entry;
}

function closePreview() {
  previewEntry.value = null;
}

function previewUrl(entry: ClipboardEntry | null): string {
  if (!entry?.image_path) return "";
  return convertFileSrc(entry.image_path);
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

const typeMeta = computed<Record<EntryType, { label: string; color: string; bg: string }>>(
  () => {
    const dark = theme.isDark;
    return {
      text: {
        label: "T",
        color: dark ? "#60a5fa" : "#2563eb",
        bg: dark ? "rgba(59, 130, 246, 0.12)" : "rgba(59, 130, 246, 0.14)",
      },
      image: {
        label: "IMG",
        color: dark ? "#34d399" : "#059669",
        bg: dark ? "rgba(16, 185, 129, 0.12)" : "rgba(16, 185, 129, 0.14)",
      },
      files: {
        label: "F",
        color: dark ? "#fbbf24" : "#d97706",
        bg: dark ? "rgba(245, 158, 11, 0.12)" : "rgba(245, 158, 11, 0.16)",
      },
    };
  }
);

onMounted(async () => {
  window.addEventListener("keydown", onGlobalKeydown);
  await store.fetchAll();
  unlistenUpdate = await listen("clipboard-update", () => {
    store.fetchAll();
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
  unlistenUpdate?.();
});
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
          ref="searchInputRef"
          v-model="store.searchQuery"
          placeholder="搜索剪贴板内容..."
          class="search-input"
          @keydown.esc.prevent="clearSearch"
        />
        <button
          v-if="!store.searchQuery"
          class="search-kbd"
          type="button"
          tabindex="-1"
          title="按 ⌘ K 聚焦搜索"
          @click="focusSearch"
        >
          ⌘ K
        </button>
        <button
          v-else
          class="search-clear"
          type="button"
          tabindex="-1"
          title="清空搜索 (Esc)"
          @click="clearSearch"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
            <path d="M6 6l12 12M18 6L6 18" stroke-linecap="round" />
          </svg>
        </button>
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

      <button
        class="clear-btn"
        type="button"
        :disabled="store.allEntries.length === 0"
        title="清空全部记录"
        @click="askClearAll"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M6 6l1 14a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-14M10 11v6M14 11v6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span>清空</span>
      </button>
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
          @click="selectIndex(idx)"
          @dblclick="onEntryDoubleClick(idx, entry)"
        >
          <button
            v-if="entry.type === 'image' && entry.thumb_path"
            class="entry-thumb"
            type="button"
            title="点击预览"
            @click.stop="openPreview(idx, entry)"
          >
            <img :src="convertFileSrc(entry.thumb_path)" alt="thumbnail" />
          </button>
          <div
            v-else
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
              @click.stop="onToggleFavorite(idx, entry)"
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
              @click.stop="onTogglePin(idx, entry)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2v8M12 10l-4 4h8l-4-4zM12 14v6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
            <button
              class="action-btn primary"
              title="复制并粘贴"
              @click.stop="onPaste(idx, entry)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="8" y="8" width="12" height="12" rx="2" />
                <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
              </svg>
            </button>
            <button
              class="action-btn danger"
              title="删除"
              @click.stop="onDelete(idx, entry)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M6 6l1 14a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-14M10 11v6M14 11v6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
          </div>
        </article>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="preview">
        <div
          v-if="previewEntry"
          class="preview-overlay"
          @click="closePreview"
        >
          <div class="preview-modal" @click.stop>
            <header class="preview-header">
              <div class="preview-meta">
                <span class="preview-title">图片预览</span>
                <span class="preview-sub">
                  {{ formatSize(previewEntry.byte_size) || "未知大小" }}
                  ·
                  {{ formatTime(previewEntry.created_at) }}
                </span>
              </div>
              <div class="preview-actions">
                <button
                  class="preview-btn"
                  title="复制到剪贴板"
                  @click="onPaste(store.selectedIndex, previewEntry)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="8" y="8" width="12" height="12" rx="2" />
                    <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
                  </svg>
                </button>
                <button
                  class="preview-btn close"
                  title="关闭 (Esc)"
                  @click="closePreview"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
                    <path d="M6 6l12 12M18 6L6 18" stroke-linecap="round" />
                  </svg>
                </button>
              </div>
            </header>
            <div class="preview-image-wrap">
              <img
                v-if="previewEntry.image_path"
                :src="previewUrl(previewEntry)"
                class="preview-image"
                alt="preview"
              />
            </div>
          </div>
        </div>
      </Transition>

      <Transition name="preview">
        <div
          v-if="showClearConfirm"
          class="confirm-overlay"
          @click="cancelClearAll"
        >
          <div class="confirm-modal" @click.stop>
            <div class="confirm-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 3 2 21h20L12 3ZM12 10v5M12 18.5h.01" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </div>
            <h3 class="confirm-title">清空全部剪贴板记录？</h3>
            <p class="confirm-desc">
              将永久删除全部 <strong>{{ store.allEntries.length }}</strong> 条记录，包括已收藏和已置顶的内容。<br />
              此操作无法撤销。
            </p>
            <div class="confirm-actions">
              <button
                class="confirm-btn cancel"
                type="button"
                @click="cancelClearAll"
              >
                取消
              </button>
              <button
                class="confirm-btn danger"
                type="button"
                autofocus
                @click="confirmClearAll"
              >
                确认清空
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
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
  gap: 14px;
  padding: 14px 22px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  background: var(--bg-toolbar);
  backdrop-filter: blur(10px) saturate(160%);
  -webkit-backdrop-filter: blur(10px) saturate(160%);
}

.search {
  position: relative;
  flex: 1 1 240px;
  min-width: 200px;
  max-width: 520px;
}

.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  width: 15px;
  height: 15px;
  color: var(--text-muted);
  pointer-events: none;
  transition: color var(--transition);
}

.search-input {
  width: 100%;
  padding: 9px 60px 9px 38px;
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

.search-input:hover {
  border-color: var(--border-strong);
}

.search-input:focus {
  border-color: var(--border-focus);
  background: var(--bg-elev-1);
  box-shadow: 0 0 0 4px var(--accent-glow);
}

.search-input:focus ~ .search-icon {
  color: var(--accent);
}

.search-kbd {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  padding: 2px 7px;
  border: 1px solid var(--border-strong);
  border-radius: 5px;
  background: var(--bg-elev-1);
  color: var(--text-muted);
  font-size: 10.5px;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: all var(--transition);
}

.search-kbd:hover {
  color: var(--text);
  border-color: var(--accent);
  background: var(--bg-active);
}

.search-clear {
  position: absolute;
  right: 6px;
  top: 50%;
  transform: translateY(-50%);
  width: 22px;
  height: 22px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: var(--bg-elev-3);
  color: var(--text-dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition);
}

.search-clear:hover {
  background: var(--danger);
  color: white;
}

.search-clear svg {
  width: 12px;
  height: 12px;
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
  padding: 6px 14px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
  white-space: nowrap;
}

.filter-tab:hover {
  color: var(--text);
}

.filter-tab.active {
  background: var(--accent);
  color: white;
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.18) inset,
    0 2px 6px var(--accent-glow);
}

.clear-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
  flex-shrink: 0;
}

.clear-btn:hover:not(:disabled) {
  border-color: var(--danger);
  color: var(--danger);
  background: rgba(239, 68, 68, 0.08);
}

.clear-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.clear-btn svg {
  width: 14px;
  height: 14px;
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
  transition: all var(--transition);
  border: 1px solid transparent;
}

.entry:hover {
  background: var(--bg-elev-1);
  border-color: var(--border);
  box-shadow: var(--shadow-sm);
}

.entry.selected {
  background: var(--bg-active);
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent-glow);
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
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
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

.entry-thumb {
  width: 42px;
  height: 42px;
  border-radius: 8px;
  border: 1px solid var(--border-strong);
  background: var(--bg-elev-2);
  padding: 0;
  cursor: pointer;
  overflow: hidden;
  flex-shrink: 0;
  position: relative;
  transition: all var(--transition);
}

.entry-thumb::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, transparent 60%, rgba(0, 0, 0, 0.35));
  opacity: 0;
  transition: opacity var(--transition);
  pointer-events: none;
}

.entry-thumb:hover {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-glow);
  transform: scale(1.04);
}

.entry-thumb:hover::after {
  opacity: 1;
}

.entry-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.preview-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.72);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
}

.preview-modal {
  width: 100%;
  max-width: 1100px;
  max-height: calc(100vh - 64px);
  background: var(--bg-elev-1);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elev-2);
  flex-shrink: 0;
}

.preview-meta {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.preview-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-bright);
}

.preview-sub {
  font-size: 11.5px;
  color: var(--text-muted);
}

.preview-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.preview-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elev-1);
  color: var(--text-dim);
  cursor: pointer;
  transition: all var(--transition);
}

.preview-btn:hover {
  color: var(--text);
  border-color: var(--border-strong);
  background: var(--bg-elev-3);
}

.preview-btn.close:hover {
  color: var(--danger);
  border-color: var(--danger);
  background: rgba(239, 68, 68, 0.12);
}

.preview-btn svg {
  width: 16px;
  height: 16px;
}

.preview-image-wrap {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    linear-gradient(45deg, rgba(255, 255, 255, 0.02) 25%, transparent 25%) 0 0 / 24px 24px,
    linear-gradient(-45deg, rgba(255, 255, 255, 0.02) 25%, transparent 25%) 0 12px / 24px 24px,
    var(--bg-base);
  overflow: auto;
  padding: 24px;
}

.preview-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow);
}

.preview-enter-from,
.preview-leave-to {
  opacity: 0;
}
.preview-enter-from .preview-modal,
.preview-leave-to .preview-modal {
  transform: scale(0.96);
}
.preview-enter-active,
.preview-leave-active {
  transition: opacity 200ms ease;
}
.preview-enter-active .preview-modal,
.preview-leave-active .preview-modal {
  transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 1100;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.confirm-modal {
  width: 100%;
  max-width: 360px;
  background: var(--bg-elev-1);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: 20px 22px 18px;
  text-align: center;
}

.confirm-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: rgba(245, 158, 11, 0.12);
  color: var(--warning);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 10px;
}

.confirm-icon svg {
  width: 20px;
  height: 20px;
}

.confirm-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-bright);
  margin-bottom: 6px;
}

.confirm-desc {
  font-size: 12.5px;
  color: var(--text-dim);
  line-height: 1.55;
  margin-bottom: 16px;
}

.confirm-desc strong {
  color: var(--text-bright);
  font-weight: 600;
}

.confirm-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.confirm-btn {
  flex: 1;
  padding: 6px 14px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}

.confirm-btn.cancel {
  border-color: var(--border-strong);
  background: var(--bg-elev-2);
  color: var(--text-dim);
}

.confirm-btn.cancel:hover {
  background: var(--bg-elev-3);
  color: var(--text);
}

.confirm-btn.danger {
  border-color: var(--danger);
  background: var(--danger);
  color: white;
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.confirm-btn.danger:hover {
  background: #dc2626;
  border-color: #dc2626;
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(239, 68, 68, 0.4);
}
</style>
