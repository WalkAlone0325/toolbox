import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ClipboardEntry, EntryType } from "../types";

export type ViewMode = "flat" | "day";

export const useClipboardStore = defineStore("clipboard", () => {
  const allEntries = ref<ClipboardEntry[]>([]);
  const loading = ref(false);
  const searchQuery = ref("");
  const filterType = ref<EntryType | null>(null);
  const selectedIndex = ref(0);
  const selectedIds = ref<Set<number>>(new Set());
  const lastSelectedId = ref<number | null>(null);
  const visibleCount = ref(100);
  const initialBatchSize = 100;
  const loadMoreStep = 100;
  const viewMode = ref<ViewMode>(
    (localStorage.getItem("sparkbox.viewMode") as ViewMode) || "flat"
  );

  function setViewMode(mode: ViewMode) {
    viewMode.value = mode;
    localStorage.setItem("sparkbox.viewMode", mode);
  }

  const hasSelection = computed(() => selectedIds.value.size > 0);
  const selectionCount = computed(() => selectedIds.value.size);
  const hasMore = computed(() => visibleCount.value < allEntries.value.length);

  const visibleEntries = computed(() =>
    allEntries.value.slice(0, visibleCount.value)
  );

  async function fetchAll() {
    loading.value = true;
    try {
      const result = await invoke<ClipboardEntry[]>("get_history", {
        query: searchQuery.value.trim(),
        filterType: filterType.value,
      });
      allEntries.value = result;
      visibleCount.value = Math.min(initialBatchSize, result.length);
      if (selectedIndex.value >= result.length) {
        selectedIndex.value = Math.max(0, result.length - 1);
      }
      const validIds = new Set(result.map((e) => e.id));
      selectedIds.value = new Set(
        [...selectedIds.value].filter((id) => validIds.has(id))
      );
    } finally {
      loading.value = false;
    }
  }

  function loadMore() {
    if (!hasMore.value) return;
    visibleCount.value = Math.min(
      visibleCount.value + loadMoreStep,
      allEntries.value.length
    );
  }

  function toggleSelect(id: number, additive: boolean, range: boolean) {
    const list = allEntries.value;
    if (range && lastSelectedId.value !== null) {
      const a = list.findIndex((e) => e.id === lastSelectedId.value);
      const b = list.findIndex((e) => e.id === id);
      if (a >= 0 && b >= 0) {
        const [lo, hi] = a <= b ? [a, b] : [b, a];
        for (let i = lo; i <= hi; i++) {
          selectedIds.value.add(list[i].id);
        }
        selectedIds.value = new Set(selectedIds.value);
        return;
      }
    }
    if (additive) {
      if (selectedIds.value.has(id)) {
        selectedIds.value.delete(id);
      } else {
        selectedIds.value.add(id);
      }
      selectedIds.value = new Set(selectedIds.value);
    } else {
      if (selectedIds.value.has(id) && selectedIds.value.size === 1) {
        selectedIds.value = new Set();
      } else {
        selectedIds.value = new Set([id]);
      }
    }
    lastSelectedId.value = id;
  }

  function selectAll() {
    selectedIds.value = new Set(allEntries.value.map((e) => e.id));
  }

  function clearSelection() {
    selectedIds.value = new Set();
    lastSelectedId.value = null;
  }

  async function deleteEntry(id: number) {
    await invoke("delete_entry", { id });
    allEntries.value = allEntries.value.filter((e) => e.id !== id);
  }

  async function deleteMany(ids: number[]) {
    if (ids.length === 0) return;
    await invoke("delete_entries", { ids });
    const set = new Set(ids);
    allEntries.value = allEntries.value.filter((e) => !set.has(e.id));
    selectedIds.value = new Set(
      [...selectedIds.value].filter((id) => !set.has(id))
    );
  }

  async function clearAll() {
    await invoke("clear_all");
    allEntries.value = [];
    selectedIndex.value = 0;
    selectedIds.value = new Set();
  }

  async function toggleFavorite(id: number) {
    await invoke("toggle_favorite", { id });
    allEntries.value = allEntries.value.map((e) =>
      e.id === id ? { ...e, fav: !e.fav } : e
    );
  }

  async function setFavoriteMany(ids: number[], fav: boolean) {
    if (ids.length === 0) return;
    await invoke("set_favorite_many", { ids, fav });
    const set = new Set(ids);
    allEntries.value = allEntries.value.map((e) =>
      set.has(e.id) ? { ...e, fav } : e
    );
  }

  async function togglePin(id: number) {
    await invoke("toggle_pin", { id });
    allEntries.value = allEntries.value.map((e) =>
      e.id === id ? { ...e, pinned: !e.pinned } : e
    );
  }

  async function setPinnedMany(ids: number[], pinned: boolean) {
    if (ids.length === 0) return;
    await invoke("set_pinned_many", { ids, pinned });
    const set = new Set(ids);
    allEntries.value = allEntries.value.map((e) =>
      set.has(e.id) ? { ...e, pinned } : e
    );
  }

  async function updateEntryMeta(id: number, tags: string | null, note: string | null) {
    await invoke("update_entry_meta", { id, tags, note });
    allEntries.value = allEntries.value.map((e) =>
      e.id === id ? { ...e, tags, note } : e
    );
  }

  function selectNext() {
    if (selectedIndex.value < allEntries.value.length - 1)
      selectedIndex.value++;
  }

  function selectPrev() {
    if (selectedIndex.value > 0) selectedIndex.value--;
  }

  return {
    allEntries,
    visibleEntries,
    loading,
    searchQuery,
    filterType,
    selectedIndex,
    selectedIds,
    hasSelection,
    selectionCount,
    hasMore,
    visibleCount,
    viewMode,
    setViewMode,
    fetchAll,
    loadMore,
    toggleSelect,
    selectAll,
    clearSelection,
    deleteEntry,
    deleteMany,
    clearAll,
    toggleFavorite,
    setFavoriteMany,
    togglePin,
    setPinnedMany,
    updateEntryMeta,
    selectNext,
    selectPrev,
  };
});
