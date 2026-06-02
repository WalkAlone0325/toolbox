<script setup lang="ts">
import { useToolboxStore } from "../stores/toolbox";

const store = useToolboxStore();
</script>

<template>
  <div class="sidebar">
    <div class="search-box">
      <input
        v-model="store.searchQuery"
        placeholder="搜索工具..."
      />
    </div>
    <div class="tool-list">
      <button
        v-for="tool in store.filteredTools"
        :key="tool.id"
        :class="{ active: tool.id === store.activeToolId }"
        @click="store.switchTool(tool.id)"
      >
        <span class="icon">{{ tool.icon }}</span>
        <span class="name">{{ tool.name }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.sidebar {
  width: 200px;
  background: #16162a;
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.search-box {
  padding: 12px;
  border-bottom: 1px solid var(--border);
}

.search-box input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: #1e1e38;
  color: var(--text);
  font-size: 13px;
  outline: none;
}

.search-box input::placeholder {
  color: var(--text-dim);
}

.tool-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tool-list button {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s;
}

.tool-list button:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.tool-list button.active {
  background: var(--bg-selected);
  color: var(--text-bright);
}

.tool-list button .icon {
  font-size: 16px;
}

.tool-list button .name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
