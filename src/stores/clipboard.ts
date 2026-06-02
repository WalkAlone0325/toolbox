import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ClipboardEntry } from "../types";

export const useClipboardStore = defineStore("clipboard", () => {
  const entries = ref<ClipboardEntry[]>([]);
  const loading = ref(false);
  const searchQuery = ref("");
  const filterType = ref<string | null>(null);
  const selectedIndex = ref(0);

  async function fetchHistory(query = "", type: string | null = null) {
    loading.value = true;
    try {
      const result = await invoke<ClipboardEntry[]>("get_history", {
        query,
        filterType: type,
      });
      entries.value = result;
      selectedIndex.value = 0;
    } finally {
      loading.value = false;
    }
  }

  async function deleteEntry(id: number) {
    await invoke("delete_entry", { id });
    entries.value = entries.value.filter((e) => e.id !== id);
  }

  async function toggleFavorite(id: number) {
    await invoke("toggle_favorite", { id });
    await fetchHistory(searchQuery.value, filterType.value);
  }

  async function togglePin(id: number) {
    await invoke("toggle_pin", { id });
    entries.value = entries.value.map((e) =>
      e.id === id ? { ...e, pinned: !e.pinned } : e
    );
  }

  function selectNext() {
    if (selectedIndex.value < entries.value.length - 1)
      selectedIndex.value++;
  }

  function selectPrev() {
    if (selectedIndex.value > 0) selectedIndex.value--;
  }

  return {
    entries,
    loading,
    searchQuery,
    filterType,
    selectedIndex,
    fetchHistory,
    deleteEntry,
    toggleFavorite,
    togglePin,
    selectNext,
    selectPrev,
  };
});
