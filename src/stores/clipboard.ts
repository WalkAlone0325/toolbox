import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ClipboardEntry, EntryType } from "../types";

export const useClipboardStore = defineStore("clipboard", () => {
  const allEntries = ref<ClipboardEntry[]>([]);
  const loading = ref(false);
  const searchQuery = ref("");
  const filterType = ref<EntryType | null>(null);
  const selectedIndex = ref(0);

  const entries = computed<ClipboardEntry[]>(() => {
    let list = allEntries.value;
    const q = searchQuery.value.trim().toLowerCase();
    if (q) {
      list = list.filter((e) => {
        if (e.text_val && e.text_val.toLowerCase().includes(q)) return true;
        if (e.source_app && e.source_app.toLowerCase().includes(q)) return true;
        if (e.file_list && e.file_list.toLowerCase().includes(q)) return true;
        return false;
      });
    }
    if (filterType.value) {
      list = list.filter((e) => e.type === filterType.value);
    }
    return list;
  });

  async function fetchAll() {
    loading.value = true;
    try {
      const result = await invoke<ClipboardEntry[]>("get_history", {
        query: "",
        filterType: null,
      });
      allEntries.value = result;
      selectedIndex.value = 0;
    } finally {
      loading.value = false;
    }
  }

  async function deleteEntry(id: number) {
    await invoke("delete_entry", { id });
    allEntries.value = allEntries.value.filter((e) => e.id !== id);
  }

  async function clearAll() {
    await invoke("clear_all");
    allEntries.value = [];
    selectedIndex.value = 0;
  }

  async function toggleFavorite(id: number) {
    await invoke("toggle_favorite", { id });
    allEntries.value = allEntries.value.map((e) =>
      e.id === id ? { ...e, fav: !e.fav } : e
    );
  }

  async function togglePin(id: number) {
    await invoke("toggle_pin", { id });
    allEntries.value = allEntries.value.map((e) =>
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
    allEntries,
    entries,
    loading,
    searchQuery,
    filterType,
    selectedIndex,
    fetchAll,
    deleteEntry,
    clearAll,
    toggleFavorite,
    togglePin,
    selectNext,
    selectPrev,
  };
});
