<script setup lang="ts">
import { computed } from "vue";
import { useToolboxStore } from "../stores/toolbox";
import { getTool } from "../tools";

const store = useToolboxStore();

const activeComponent = computed(() => {
  const tool = getTool(store.activeToolId);
  return tool?.component ?? null;
});

const activeTool = computed(() => getTool(store.activeToolId));
</script>

<template>
  <main class="tool-container">
    <header v-if="activeTool" class="tool-header">
      <div class="header-left">
        <span class="header-icon">{{ activeTool.icon }}</span>
        <div>
          <h1 class="header-title">{{ activeTool.name }}</h1>
          <p class="header-desc">{{ activeTool.desc }}</p>
        </div>
      </div>
    </header>
    <div class="tool-body">
      <component :is="activeComponent" />
    </div>
  </main>
</template>

<style scoped>
.tool-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
  min-width: 0;
}

.tool-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
  background: rgba(17, 20, 27, 0.6);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.header-icon {
  font-size: 22px;
  width: 36px;
  height: 36px;
  border-radius: var(--radius);
  background: linear-gradient(135deg, var(--bg-elev-3), var(--bg-elev-2));
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-bright);
  letter-spacing: -0.01em;
  margin: 0;
}

.header-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  margin-top: 1px;
}

.tool-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
