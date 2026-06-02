import { onMounted } from "vue";
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
