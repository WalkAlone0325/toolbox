import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { tools } from "../tools";

export const useSparkboxStore = defineStore("sparkbox", () => {
  const activeToolId = ref(tools[0]?.id ?? "");
  const searchQuery = ref("");

  const activeTool = computed(() =>
    tools.find((t) => t.id === activeToolId.value)
  );

  const filteredTools = computed(() => {
    if (!searchQuery.value) return tools;
    const q = searchQuery.value.toLowerCase();
    return tools.filter(
      (t) =>
        t.name.toLowerCase().includes(q) ||
        t.keywords.some((k) => k.includes(q)) ||
        t.desc.toLowerCase().includes(q)
    );
  });

  function switchTool(id: string) {
    activeToolId.value = id;
    searchQuery.value = "";
  }

  return {
    activeToolId,
    activeTool,
    searchQuery,
    filteredTools,
    switchTool,
  };
});
