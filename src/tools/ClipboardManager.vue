<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, watch } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useClipboardStore } from "../stores/clipboard";
import { useThemeStore } from "../stores/theme";
import { useToastStore } from "../stores/toast";
import type { ClipboardEntry, EntryType } from "../types";
import ContextMenu, { type ContextMenuEntry } from "../components/ContextMenu.vue";

const store = useClipboardStore();
const theme = useThemeStore();
const toast = useToastStore();
const searchInputRef = ref<HTMLInputElement>();
const previewEntry = ref<ClipboardEntry | null>(null);
const showClearConfirm = ref(false);
let unlistenUpdate: UnlistenFn | undefined;
let unlistenBlur: UnlistenFn | undefined;
let unlistenShown: UnlistenFn | undefined;
let searchTimer: ReturnType<typeof setTimeout> | undefined;

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
  const isMod = e.metaKey || e.ctrlKey;
  const isModK = isMod && e.key.toLowerCase() === "k";
  const isModF = isMod && e.key.toLowerCase() === "f";
  if (isModK || isModF) {
    e.preventDefault();
    focusSearch();
    return;
  }
  if (isMod && ["1", "2", "3", "4"].includes(e.key)) {
    e.preventDefault();
    const idx = parseInt(e.key, 10);
    const tabs: (EntryType | null)[] = [null, "text", "image", "files"];
    setFilter(tabs[idx - 1] ?? null);
    return;
  }
  if (isMod && e.key.toLowerCase() === "a") {
    e.preventDefault();
    store.selectAll();
    return;
  }
  if (e.key === "Escape") {
    if (aiResult.value) {
      e.preventDefault();
      closeAi();
      return;
    }
    if (store.hasSelection) {
      e.preventDefault();
      store.clearSelection();
      return;
    }
    if (ctxMenu.value) {
      e.preventDefault();
      closeContextMenu();
      return;
    }
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
    return;
  }
  const active = document.activeElement;
  const inInput = active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement;
  if (inInput) return;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    store.selectNext();
    return;
  }
  if (e.key === "ArrowUp") {
    e.preventDefault();
    store.selectPrev();
    return;
  }
  if (e.key === "Enter") {
    const entry = entries.value[store.selectedIndex];
    if (entry) {
      e.preventDefault();
      onPaste(store.selectedIndex, entry);
    }
    return;
  }
  if (e.key === "Delete" || e.key === "Backspace") {
    const entry = entries.value[store.selectedIndex];
    if (entry) {
      e.preventDefault();
      onDelete(store.selectedIndex, entry);
    }
  }
}

function selectIndex(idx: number) {
  if (idx >= 0 && idx < entries.value.length) {
    store.selectedIndex = idx;
  }
}

function onEntryClick(e: MouseEvent, idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  if (e.metaKey || e.ctrlKey || e.shiftKey || store.hasSelection) {
    store.toggleSelect(entry.id, e.metaKey || e.ctrlKey, e.shiftKey);
  }
}

async function onBulkFavorite(fav: boolean) {
  const ids = [...store.selectedIds];
  try {
    await store.setFavoriteMany(ids, fav);
    toast.success(fav ? "已加入收藏" : "已取消收藏");
  } catch {
    toast.danger("操作失败");
  }
}

async function onBulkPin(pinned: boolean) {
  const ids = [...store.selectedIds];
  try {
    await store.setPinnedMany(ids, pinned);
    toast.success(pinned ? "已置顶" : "已取消置顶");
  } catch {
    toast.danger("操作失败");
  }
}

async function onBulkDelete() {
  const ids = [...store.selectedIds];
  if (ids.length === 0) return;
  if (!window.confirm(`确定删除选中的 ${ids.length} 条记录？`)) return;
  try {
    await store.deleteMany(ids);
    toast.warning(`已删除 ${ids.length} 条`);
  } catch {
    toast.danger("删除失败");
  }
}

async function onBulkExport() {
  const ids = new Set(store.selectedIds);
  const list = entries.value.filter((e) => ids.has(e.id));
  await exportEntriesToFile(list);
}

async function onExportAll() {
  await exportEntriesToFile(entries.value);
}

function onImport() {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".json,application/json";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const data = JSON.parse(text);
      if (!Array.isArray(data)) throw new Error("invalid format");
      const count = await invoke<number>("import_history", { entries: data });
      toast.success(`已导入 ${count} 条`);
      await store.fetchAll();
    } catch (e) {
      toast.danger("导入失败：" + (e as Error).message);
    }
  };
  input.click();
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

async function exportEntriesToFile(list: ClipboardEntry[]) {
  if (list.length === 0) {
    toast.info("没有可导出的条目");
    return;
  }
  try {
    const payload = list.map((e) => ({
      type: e.type,
      text_val: e.text_val,
      file_list: e.file_list,
      source_app: e.source_app,
      byte_size: e.byte_size,
      created_at: e.created_at,
      last_used_at: e.last_used_at,
      fav: e.fav,
      pinned: e.pinned,
    }));
    const ts = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
    const blob = new Blob([JSON.stringify(payload, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `sparkbox-export-${ts}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    toast.success(`已导出 ${list.length} 条`);
  } catch (e) {
    toast.danger("导出失败");
  }
}

async function onPaste(idx: number, entry: ClipboardEntry) {
  selectIndex(idx);
  try {
    await invoke("paste_entry", { id: entry.id });
    if (entry.type === "text" && entry.text_val) {
      toast.preview(entry.text_val, "success");
    } else if (entry.type === "image") {
      toast.success(`已复制图片 · ${formatSize(entry.byte_size) || "未知大小"}`);
    } else if (entry.type === "files") {
      const files = parseFiles(entry.file_list || "");
      toast.success(`已复制 ${files.length} 个文件`);
    } else {
      toast.success("已复制到剪贴板");
    }
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

function parseFiles(fileList: string): string[] {
  try {
    const arr = JSON.parse(fileList);
    if (Array.isArray(arr)) return arr as string[];
  } catch {
    // ignore
  }
  return [];
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

function dayKey(ts: number): string {
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`;
}

function dayLabel(ts: number): string {
  const d = new Date(ts * 1000);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const target = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  const diff = Math.floor((today.getTime() - target.getTime()) / 86400000);
  if (diff === 0) return "今天";
  if (diff === 1) return "昨天";
  if (diff === 2) return "前天";
  if (diff < 7 && diff > 0) return `${diff} 天前`;
  return `${d.getFullYear()}年${d.getMonth() + 1}月${d.getDate()}日`;
}

const entries = computed(() => store.visibleEntries);

const separatorBefore = computed<Map<number, string>>(() => {
  const map = new Map<number, string>();
  const list = entries.value;
  if (list.length === 0) return map;

  if (store.viewMode === "flat") {
    let pinStart = -1;
    let normalStart = -1;
    for (let i = 0; i < list.length; i++) {
      if (pinStart === -1 && list[i].pinned) pinStart = i;
      if (normalStart === -1 && !list[i].pinned && pinStart !== -1) {
        normalStart = i;
        break;
      }
    }
    if (pinStart === 0 && normalStart > 0) {
      map.set(0, "已置顶");
    }
    return map;
  }

  let lastDay = "";
  let pinSeparatorPlaced = false;
  for (let i = 0; i < list.length; i++) {
    const entry = list[i];
    if (entry.pinned) {
      if (!pinSeparatorPlaced) {
        map.set(i, "已置顶");
        pinSeparatorPlaced = true;
      }
      continue;
    }
    const key = dayKey(entry.last_used_at || entry.created_at);
    if (key !== lastDay) {
      map.set(i, dayLabel(entry.last_used_at || entry.created_at));
      lastDay = key;
    }
  }
  return map;
});

const showPreviewPane = ref(false);

const selectedEntry = computed<ClipboardEntry | null>(
  () => entries.value[store.selectedIndex] ?? null
);

const paneTagsInput = ref("");
const paneNoteInput = ref("");

watch(selectedEntry, (entry) => {
  paneTagsInput.value = entry?.tags ?? "";
  paneNoteInput.value = entry?.note ?? "";
}, { immediate: true });

async function saveEntryMeta() {
  if (!selectedEntry.value) return;
  const id = selectedEntry.value.id;
  const tags = paneTagsInput.value.trim();
  const note = paneNoteInput.value.trim();
  try {
    await store.updateEntryMeta(id, tags || null, note || null);
  } catch {
    toast.danger("保存失败");
  }
}

function togglePreviewPane() {
  showPreviewPane.value = !showPreviewPane.value;
}

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
  unlistenBlur = await listen("sparkbox-window-blur", () => {
    if (autoHideOnBlur.value) {
      invoke("hide_main_window").catch(() => {});
    }
  });
  unlistenShown = await listen("sparkbox-window-shown", () => {
    focusSearch();
  });
  requestAnimationFrame(() => setupObserver());
});

onUnmounted(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
  window.removeEventListener("sparkbox-settings-changed", refreshSettings);
  unlistenUpdate?.();
  unlistenBlur?.();
  unlistenShown?.();
  if (searchTimer) clearTimeout(searchTimer);
  loadMoreObserver?.disconnect();
});

const autoHideOnBlur = ref(localStorage.getItem("sparkbox.autoHideOnBlur") === "1");

function refreshSettings() {
  autoHideOnBlur.value = localStorage.getItem("sparkbox.autoHideOnBlur") === "1";
}

window.addEventListener("sparkbox-settings-changed", refreshSettings);

const ctxMenu = ref<{ x: number; y: number; entry: ClipboardEntry; items: ContextMenuEntry[] } | null>(null);

function buildCtxItems(entry: ClipboardEntry): ContextMenuEntry[] {
  const items: ContextMenuEntry[] = [
    { id: "paste", label: "复制到剪贴板" },
    { id: "copy-text", label: "复制为纯文本", disabled: entry.type !== "text" },
  ];
  items.push({ id: "sep-1", separator: true });
  items.push({
    id: "ai-translate",
    label: "AI · 翻译为英文",
    disabled: entry.type !== "text",
  });
  items.push({
    id: "ai-translate-zh",
    label: "AI · 翻译为中文",
    disabled: entry.type !== "text",
  });
  items.push({
    id: "ai-summarize",
    label: "AI · 总结要点",
    disabled: entry.type !== "text",
  });
  items.push({
    id: "ai-polish",
    label: "AI · 润色",
    disabled: entry.type !== "text",
  });
  items.push({
    id: "ai-extract",
    label: "AI · 提取关键信息",
    disabled: entry.type !== "text",
  });
  items.push({
    id: "ai-explain",
    label: "AI · 解释",
    disabled: entry.type !== "text",
  });
  items.push({ id: "sep-ai", separator: true });
  items.push({
    id: "fav",
    label: entry.fav ? "取消收藏" : "收藏",
  });
  items.push({
    id: "pin",
    label: entry.pinned ? "取消置顶" : "置顶",
  });
  items.push({ id: "sep-2", separator: true });
  items.push({ id: "delete", label: "删除", danger: true });
  return items;
}

function openContextMenu(e: MouseEvent, entry: ClipboardEntry) {
  e.preventDefault();
  const idx = entries.value.findIndex((x) => x.id === entry.id);
  if (idx >= 0) store.selectedIndex = idx;
  ctxMenu.value = {
    x: e.clientX,
    y: e.clientY,
    entry,
    items: buildCtxItems(entry),
  };
}

async function onCtxSelect(id: string) {
  const target = ctxMenu.value?.entry;
  if (!target) return;
  const idx = entries.value.findIndex((x) => x.id === target.id);
  if (idx < 0) return;
  if (id.startsWith("ai-")) {
    const action = id.slice(3);
    await startAiTransform(action, target);
    return;
  }
  switch (id) {
    case "paste":
      await onPaste(idx, target);
      break;
    case "copy-text":
      if (target.text_val) {
        try {
          await navigator.clipboard.writeText(target.text_val);
          toast.preview(target.text_val, "success");
        } catch {
          toast.danger("复制失败");
        }
      }
      break;
    case "fav":
      await onToggleFavorite(idx, target);
      break;
    case "pin":
      await onTogglePin(idx, target);
      break;
    case "delete":
      await onDelete(idx, target);
      break;
  }
}

interface AIResultState {
  action: string;
  source: string;
  result: string;
  loading: boolean;
  error: string | null;
  taskId: string | null;
}

const aiResult = ref<AIResultState | null>(null);

const actionLabels: Record<string, string> = {
  translate: "翻译为英文",
  "translate_zh": "翻译为中文",
  summarize: "总结要点",
  polish: "润色",
  extract: "提取关键信息",
  explain: "解释",
  rewrite_formal: "改写为正式",
  rewrite_casual: "改写为口语",
  custom: "AI 处理",
};

async function startAiTransform(action: string, entry: ClipboardEntry) {
  if (!entry.text_val) {
    toast.warning("仅支持文本内容");
    return;
  }
  aiResult.value = {
    action,
    source: entry.text_val,
    result: "",
    loading: true,
    error: null,
    taskId: null,
  };
  try {
    const { Channel, invoke } = await import("@tauri-apps/api/core");
    type AIEvent =
      | { type: "chunk"; text: string }
      | { type: "done"; text: string }
      | { type: "error"; message: string };
    const channel = new Channel<AIEvent>();
    channel.onmessage = (msg) => {
      if (!aiResult.value) return;
      if (msg.type === "chunk") {
        aiResult.value.result += msg.text;
      } else if (msg.type === "done") {
        aiResult.value.result = msg.text || aiResult.value.result;
        aiResult.value.loading = false;
        aiResult.value.taskId = null;
      } else if (msg.type === "error") {
        aiResult.value.error = msg.message;
        aiResult.value.loading = false;
        aiResult.value.taskId = null;
      }
    };
    const taskId = await invoke<string>("ai_transform", {
      input: { action, content: entry.text_val },
      onEvent: channel,
    });
    if (aiResult.value) aiResult.value.taskId = taskId;
  } catch (e) {
    if (aiResult.value) {
      aiResult.value.error = (e as Error).message || String(e);
      aiResult.value.loading = false;
    }
  }
}

async function cancelAi() {
  const id = aiResult.value?.taskId;
  if (!id) return;
  try {
    await import("@tauri-apps/api/core").then(({ invoke }) =>
      invoke("ai_cancel", { taskId: id })
    );
  } catch {
    // ignore
  }
  if (aiResult.value) {
    aiResult.value.loading = false;
    aiResult.value.taskId = null;
  }
}

function closeAi() {
  if (aiResult.value?.loading) {
    cancelAi();
  }
  aiResult.value = null;
}

async function copyAiResult() {
  if (!aiResult.value?.result) return;
  try {
    await navigator.clipboard.writeText(aiResult.value.result);
    toast.success("已复制 AI 结果");
  } catch {
    toast.danger("复制失败");
  }
}

async function replaceWithAiResult() {
  if (!aiResult.value?.result || !aiResult.value?.source) return;
  try {
    await navigator.clipboard.writeText(aiResult.value.result);
    toast.success("已替换并复制到剪贴板");
  } catch {
    toast.danger("复制失败");
  }
}

function closeContextMenu() {
  ctxMenu.value = null;
}

watch(
  () => [store.searchQuery, store.filterType],
  () => {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      store.fetchAll();
    }, 200);
  }
);

const sentinelRef = ref<HTMLDivElement | null>(null);
let loadMoreObserver: IntersectionObserver | null = null;

function setupObserver() {
  if (loadMoreObserver) loadMoreObserver.disconnect();
  if (!sentinelRef.value) return;
  loadMoreObserver = new IntersectionObserver(
    (entriesObs) => {
      for (const item of entriesObs) {
        if (item.isIntersecting && store.hasMore) {
          store.loadMore();
        }
      }
    },
    { rootMargin: "200px" }
  );
  loadMoreObserver.observe(sentinelRef.value);
}

watch(sentinelRef, () => setupObserver());
watch(
  () => store.allEntries.length,
  () => {
    requestAnimationFrame(() => setupObserver());
  }
);
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

      <button
        class="clear-btn"
        type="button"
        title="导出全部为 JSON"
        @click="onExportAll"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 3v12M7 10l5 5 5-5M5 21h14" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span>导出</span>
      </button>

      <button
        class="clear-btn"
        type="button"
        title="从 JSON 导入"
        @click="onImport"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 21V9M7 14l5-5 5 5M5 3h14" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span>导入</span>
      </button>

      <button
        class="clear-btn"
        type="button"
        :class="{ active: showPreviewPane }"
        title="切换预览侧栏"
        @click="togglePreviewPane"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="4" width="18" height="16" rx="2" />
          <path d="M14 4v16" />
        </svg>
        <span>预览</span>
      </button>

      <button
        class="clear-btn"
        type="button"
        :class="{ active: store.viewMode === 'day' }"
        :title="store.viewMode === 'day' ? '切换为列表视图' : '切换为按天分组'"
        @click="store.setViewMode(store.viewMode === 'day' ? 'flat' : 'day')"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="5" width="18" height="3" rx="1" />
          <rect x="3" y="11" width="18" height="3" rx="1" />
          <rect x="3" y="17" width="18" height="3" rx="1" />
        </svg>
        <span>{{ store.viewMode === 'day' ? '分组' : '列表' }}</span>
      </button>
    </div>

    <Transition name="bar">
      <div v-if="store.hasSelection" class="bulk-bar">
        <span class="bulk-count">已选 {{ store.selectionCount }} 项</span>
        <button class="bulk-btn" @click="onBulkFavorite(true)">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 17.27 18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/></svg>
          收藏
        </button>
        <button class="bulk-btn" @click="onBulkFavorite(false)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 17.27 18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" stroke-linejoin="round" /></svg>
          取消收藏
        </button>
        <button class="bulk-btn" @click="onBulkPin(true)">置顶</button>
        <button class="bulk-btn" @click="onBulkPin(false)">取消置顶</button>
        <button class="bulk-btn" @click="onBulkExport">导出</button>
        <button class="bulk-btn danger" @click="onBulkDelete">删除</button>
        <button class="bulk-btn ghost" @click="store.clearSelection()">取消</button>
      </div>
    </Transition>

    <div class="list-wrap">
      <div v-if="store.loading" class="state loading">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <div v-else-if="entries.length === 0" class="state empty">
        <svg viewBox="0 0 64 64" fill="none" class="empty-icon">
          <rect x="14" y="18" width="36" height="32" rx="6" stroke="currentColor" stroke-width="2.5" />
          <rect x="24" y="12" width="16" height="8" rx="2" stroke="currentColor" stroke-width="2.5" />
          <path d="M24 34h16M24 42h10" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
        </svg>
        <p class="empty-title">暂无剪贴板记录</p>
        <p class="empty-hint">复制一些内容后会自动出现在这里</p>
      </div>

      <div v-else class="entries-wrap">
        <div class="entries">
          <template v-for="(entry, idx) in entries" :key="entry.id">
            <div
              v-if="separatorBefore.has(idx)"
              class="pin-sep"
            >
              <span>{{ separatorBefore.get(idx) }}</span>
            </div>
            <article
              class="entry"
              :class="{
                selected: idx === store.selectedIndex,
                checked: store.selectedIds.has(entry.id),
                pinned: entry.pinned,
              }"
              @click="onEntryClick($event, idx, entry)"
              @dblclick="onEntryDoubleClick(idx, entry)"
              @contextmenu="openContextMenu($event, entry)"
            >
          <button
            v-if="store.hasSelection"
            class="entry-check"
            :class="{ checked: store.selectedIds.has(entry.id) }"
            type="button"
            tabindex="-1"
            @click.stop="store.toggleSelect(entry.id, true, false)"
            :title="store.selectedIds.has(entry.id) ? '取消选择' : '选中'"
          >
            <svg v-if="store.selectedIds.has(entry.id)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <path d="m5 12 5 5 9-11" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>

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
          </template>
          <div v-if="store.hasMore" ref="sentinelRef" class="load-more-sentinel">
            <span>加载更多（剩 {{ store.allEntries.length - store.visibleCount }} 条）</span>
          </div>
        </div>
      </div>
    </div>

    <aside v-if="showPreviewPane" class="preview-pane">
      <header class="pane-header">
        <span class="pane-title">预览</span>
        <button class="pane-close" type="button" @click="togglePreviewPane" title="关闭">×</button>
      </header>
      <div class="pane-body">
        <div v-if="!selectedEntry" class="pane-empty">未选中条目</div>
        <template v-else>
          <div v-if="selectedEntry.type === 'image' && selectedEntry.image_path" class="pane-image-wrap">
            <img :src="convertFileSrc(selectedEntry.image_path)" alt="preview" />
          </div>
          <pre v-else-if="selectedEntry.type === 'text'" class="pane-text">{{ selectedEntry.text_val }}</pre>
          <div v-else-if="selectedEntry.type === 'files' && selectedEntry.file_list" class="pane-files">
            <div v-for="(f, i) in parseFiles(selectedEntry.file_list)" :key="i" class="pane-file-item">
              {{ f }}
            </div>
          </div>
          <div class="pane-meta">
            <div class="pane-meta-row"><span>类型</span><strong>{{ selectedEntry.type }}</strong></div>
            <div class="pane-meta-row"><span>大小</span><strong>{{ formatSize(selectedEntry.byte_size) }}</strong></div>
            <div class="pane-meta-row"><span>来源</span><strong>{{ selectedEntry.source_app || '-' }}</strong></div>
            <div class="pane-meta-row"><span>时间</span><strong>{{ formatTime(selectedEntry.created_at) }}</strong></div>
            <div class="pane-meta-row"><span>使用</span><strong>{{ selectedEntry.use_count }} 次</strong></div>
          </div>

          <div class="pane-tags">
            <label class="pane-field-label">标签</label>
            <input
              v-model="paneTagsInput"
              type="text"
              placeholder="用逗号分隔，如：work, 重要"
              class="field-input"
              @blur="saveEntryMeta()"
            />
          </div>

          <div class="pane-note">
            <label class="pane-field-label">备注</label>
            <textarea
              v-model="paneNoteInput"
              rows="3"
              placeholder="给这条记录加一句备注..."
              class="field-input pane-note-area"
              @blur="saveEntryMeta()"
            />
          </div>
        </template>
      </div>
    </aside>

    <Teleport to="body">
      <ContextMenu
        v-if="ctxMenu"
        :x="ctxMenu.x"
        :y="ctxMenu.y"
        :items="ctxMenu.items"
        @select="onCtxSelect"
        @close="closeContextMenu"
      />
    </Teleport>

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

      <Transition name="preview">
        <div v-if="aiResult" class="ai-overlay" @click.self="closeAi">
          <div class="ai-modal">
            <header class="ai-header">
              <div class="ai-title">
                <span class="ai-badge">AI</span>
                {{ actionLabels[aiResult.action] || "AI 处理" }}
              </div>
              <button class="ai-close" type="button" @click="closeAi" title="关闭">×</button>
            </header>
            <div class="ai-body">
              <section class="ai-section">
                <div class="ai-section-title">原文</div>
                <pre class="ai-section-text">{{ aiResult.source }}</pre>
              </section>
              <section class="ai-section">
                <div class="ai-section-title">
                  结果
                  <span v-if="aiResult.loading" class="ai-loading-dot">生成中…</span>
                </div>
                <pre v-if="aiResult.result || !aiResult.loading" class="ai-section-text result">{{ aiResult.result || "（无输出）" }}</pre>
                <div v-else class="ai-skeleton">
                  <span></span><span></span><span></span>
                </div>
                <div v-if="aiResult.error" class="ai-error">{{ aiResult.error }}</div>
              </section>
            </div>
            <footer class="ai-footer">
              <button
                class="confirm-btn cancel"
                type="button"
                :disabled="!aiResult.result && !aiResult.loading"
                @click="copyAiResult"
              >
                复制结果
              </button>
              <button
                class="confirm-btn cancel"
                type="button"
                :disabled="!aiResult.result"
                @click="replaceWithAiResult"
              >
                替换剪贴板
              </button>
              <button
                v-if="aiResult.loading"
                class="confirm-btn cancel"
                type="button"
                @click="cancelAi"
              >
                停止
              </button>
              <button class="confirm-btn danger" type="button" @click="closeAi">关闭</button>
            </footer>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.clipboard-tool {
  display: grid;
  grid-template-columns: 1fr auto;
  grid-template-rows: auto auto 1fr;
  height: 100%;
  background: var(--bg-base);
}

.clipboard-tool > .toolbar {
  grid-column: 1;
  grid-row: 1;
}

.clipboard-tool > .bulk-bar {
  grid-column: 1;
  grid-row: 2;
}

.clipboard-tool > .list-wrap {
  grid-column: 1;
  grid-row: 3;
  min-height: 0;
}

.clipboard-tool > .preview-pane {
  grid-column: 2;
  grid-row: 1 / -1;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 18px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  background: var(--bg-toolbar);
  backdrop-filter: blur(10px) saturate(160%);
  -webkit-backdrop-filter: blur(10px) saturate(160%);
  flex-wrap: wrap;
}

.toolbar > .search {
  flex: 1 1 200px;
  min-width: 180px;
  max-width: 520px;
  order: 1;
}

.toolbar > .filters {
  order: 2;
}

.toolbar > .clear-btn {
  order: 3;
}

@media (max-width: 720px) {
  .toolbar {
    padding: 10px 12px;
    gap: 8px;
  }
  .toolbar > .search {
    order: 1;
    flex-basis: 100%;
    max-width: none;
  }
  .toolbar > .filters {
    order: 2;
  }
  .toolbar > .clear-btn {
    order: 3;
    margin-left: auto;
    padding: 6px 8px;
  }
  .toolbar > .clear-btn span {
    display: none;
  }
  .filter-tab {
    padding: 5px 10px;
    font-size: 11.5px;
  }
}

.search {
  position: relative;
  flex: 1 1 200px;
  min-width: 180px;
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

.bulk-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 22px;
  background: var(--bg-elev-2);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  overflow-x: auto;
}

.bulk-count {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-bright);
  margin-right: 6px;
  white-space: nowrap;
}

.bulk-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elev-1);
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
  white-space: nowrap;
}

.bulk-btn:hover {
  border-color: var(--border-strong);
  color: var(--text);
}

.bulk-btn svg {
  width: 12px;
  height: 12px;
}

.bulk-btn.danger {
  color: var(--danger);
  border-color: rgba(239, 68, 68, 0.4);
}

.bulk-btn.danger:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--danger);
}

.bulk-btn.ghost {
  margin-left: auto;
}

.bar-enter-from,
.bar-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.bar-enter-active,
.bar-leave-active {
  transition: opacity 150ms ease, transform 150ms ease;
}

.entry-check {
  width: 18px;
  height: 18px;
  border: 1.5px solid var(--border-strong);
  border-radius: 4px;
  background: var(--bg-elev-1);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all var(--transition);
  padding: 0;
}

.entry-check:hover {
  border-color: var(--accent);
}

.entry-check.checked {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.entry-check svg {
  width: 12px;
  height: 12px;
}

.entry.checked {
  background: var(--bg-active);
}

.pin-sep {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px 4px;
  font-size: 10.5px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.pin-sep::after {
  content: "";
  flex: 1;
  height: 1px;
  background: var(--border);
}

.entries-wrap {
  padding: 8px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.preview-pane {
  width: 320px;
  border-left: 1px solid var(--border);
  background: var(--bg-elev-1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

@media (max-width: 900px) {
  .clipboard-tool {
    grid-template-columns: 1fr;
  }
  .clipboard-tool > .preview-pane {
    grid-column: 1;
    grid-row: 1 / -1;
    position: fixed;
    right: 12px;
    top: 12px;
    bottom: 12px;
    width: min(320px, calc(100vw - 24px));
    z-index: 500;
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
  }
}

.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-toolbar);
}

.pane-title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-bright);
  letter-spacing: 0.04em;
}

.pane-close {
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
}

.pane-close:hover {
  background: var(--bg-elev-3);
  color: var(--text);
}

.pane-body {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
}

.pane-empty {
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
  padding: 40px 12px;
}

.pane-text {
  flex: 1;
  margin: 0;
  padding: 10px 12px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text-bright);
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 60vh;
  overflow-y: auto;
}

.pane-image-wrap {
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pane-image-wrap img {
  max-width: 100%;
  max-height: 60vh;
  object-fit: contain;
  border-radius: var(--radius-sm);
}

.pane-files {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 10px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.pane-file-item {
  font-size: 12px;
  color: var(--text);
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  word-break: break-all;
}

.pane-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  font-size: 11.5px;
}

.pane-meta-row {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  color: var(--text-muted);
}

.pane-meta-row strong {
  color: var(--text-bright);
  font-weight: 500;
}

.pane-tags,
.pane-note {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pane-field-label {
  font-size: 11.5px;
  color: var(--text-muted);
  font-weight: 500;
}

.pane-note-area {
  resize: vertical;
  min-height: 60px;
  font-family: inherit;
  line-height: 1.5;
}

.clear-btn.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--bg-active);
}

.load-more-sentinel {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 14px;
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 11.5px;
  border: 1px dashed var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-1);
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
  gap: var(--row-gap, 4px);
}

.entry {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: var(--row-padding-y, 10px) 12px var(--row-padding-y, 10px) 10px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: all var(--transition);
  border: 1px solid transparent;
  min-height: var(--entry-min-height, 44px);
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

.ai-overlay {
  position: fixed;
  inset: 0;
  z-index: 1200;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.ai-modal {
  width: 100%;
  max-width: 720px;
  max-height: calc(100vh - 48px);
  background: var(--bg-elev-1);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ai-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elev-2);
}

.ai-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-bright);
}

.ai-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  color: white;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.06em;
}

.ai-close {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
}

.ai-close:hover {
  background: var(--bg-elev-3);
  color: var(--text);
}

.ai-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.ai-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ai-section-title {
  font-size: 11.5px;
  color: var(--text-muted);
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ai-loading-dot {
  font-size: 11px;
  color: var(--accent);
  text-transform: none;
  letter-spacing: 0;
  font-weight: 500;
}

.ai-loading-dot::before {
  content: "●";
  margin-right: 4px;
  animation: pulse 1.2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.ai-section-text {
  margin: 0;
  padding: 10px 12px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text);
  font-family: inherit;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 220px;
  overflow-y: auto;
}

.ai-section-text.result {
  color: var(--text-bright);
  background: var(--bg-elev-1);
  border-color: var(--accent-glow);
}

.ai-skeleton {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
}

.ai-skeleton span {
  display: block;
  height: 12px;
  border-radius: 6px;
  background: linear-gradient(90deg, var(--bg-elev-2) 0%, var(--bg-elev-3) 50%, var(--bg-elev-2) 100%);
  background-size: 200% 100%;
  animation: shimmer 1.4s linear infinite;
}

.ai-skeleton span:nth-child(1) { width: 80%; }
.ai-skeleton span:nth-child(2) { width: 92%; }
.ai-skeleton span:nth-child(3) { width: 60%; }

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.ai-error {
  margin-top: 6px;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  background: rgba(239, 68, 68, 0.1);
  color: var(--danger);
  border: 1px solid rgba(239, 68, 68, 0.3);
  font-size: 12px;
}

.ai-footer {
  display: flex;
  gap: 8px;
  padding: 10px 16px;
  border-top: 1px solid var(--border);
  background: var(--bg-elev-2);
  justify-content: flex-end;
}

.ai-footer .confirm-btn {
  flex: none;
  padding: 7px 14px;
  font-size: 12.5px;
}
</style>
