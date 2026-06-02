import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted } from "vue";
import { useClipboardStore } from "../stores/clipboard";

export function useClipboardMonitor() {
  const store = useClipboardStore();

  function handleUpdate() {
    store.fetchHistory(store.searchQuery, store.filterType);
  }

  onMounted(async () => {
    await store.fetchHistory();
  });

  return { refresh: handleUpdate };
}
