import { onMounted } from "vue";
import { useClipboardStore } from "../stores/clipboard";

export function useClipboardMonitor() {
  const store = useClipboardStore();

  function handleUpdate() {
    store.fetchAll();
  }

  onMounted(async () => {
    await store.fetchAll();
  });

  return { refresh: handleUpdate };
}
