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
        <span class="header-icon">
          <span class="header-icon-emoji">{{ activeTool.icon }}</span>
        </span>
        <div class="header-text">
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
  padding: 14px 22px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-header);
  backdrop-filter: blur(14px) saturate(180%);
  -webkit-backdrop-filter: blur(14px) saturate(180%);
  flex-shrink: 0;
  position: relative;
  z-index: 1;
}

.tool-header::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    var(--accent-glow),
    transparent
  );
  pointer-events: none;
  opacity: 0.5;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.header-icon {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: var(--radius);
  background: linear-gradient(135deg, var(--accent), var(--accent-hover));
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.15) inset,
    0 4px 10px var(--accent-glow);
}

.header-icon-emoji {
  font-size: 20px;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.15));
  line-height: 1;
}

.header-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-bright);
  letter-spacing: -0.01em;
  margin: 0;
  line-height: 1.3;
}

.header-desc {
  font-size: 11.5px;
  color: var(--text-dim);
  margin: 0;
  line-height: 1.4;
}

.tool-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
