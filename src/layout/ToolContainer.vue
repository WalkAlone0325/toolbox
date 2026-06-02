<script setup lang="ts">
import { computed } from "vue";
import { useToolboxStore } from "../stores/toolbox";
import { getTool } from "../tools";

const store = useToolboxStore();

const activeComponent = computed(() => {
  const tool = getTool(store.activeToolId);
  return tool?.component ?? null;
});
</script>

<template>
  <div class="tool-container">
    <component :is="activeComponent" />
  </div>
</template>

<style scoped>
.tool-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
