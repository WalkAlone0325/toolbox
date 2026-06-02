<script setup lang="ts">
import { useToolboxStore } from "../stores/toolbox";

const store = useToolboxStore();
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-logo">
        <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="2" y="2" width="28" height="28" rx="8" fill="url(#bg)" />
          <path
            d="M11 10h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H11a2 2 0 0 1-2-2V12a2 2 0 0 1 2-2z"
            stroke="white"
            stroke-width="1.8"
            stroke-linejoin="round"
          />
          <rect x="13" y="8" width="6" height="4" rx="1.2" fill="white" />
          <path d="M12 17h8M12 20h5" stroke="white" stroke-width="1.6" stroke-linecap="round" />
          <defs>
            <linearGradient id="bg" x1="0" y1="0" x2="32" y2="32" gradientUnits="userSpaceOnUse">
              <stop stop-color="#818cf8" />
              <stop offset="1" stop-color="#6366f1" />
            </linearGradient>
          </defs>
        </svg>
      </div>
      <div class="brand-text">
        <div class="brand-name">Toolbox</div>
        <div class="brand-sub">工具箱</div>
      </div>
    </div>

    <div class="search-box">
      <svg
        class="search-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" stroke-linecap="round" />
      </svg>
      <input
        v-model="store.searchQuery"
        placeholder="搜索工具..."
        class="search-input"
      />
    </div>

    <div class="section-label">工具</div>

    <nav class="tool-list">
      <button
        v-for="tool in store.filteredTools"
        :key="tool.id"
        class="tool-item"
        :class="{ active: tool.id === store.activeToolId }"
        @click="store.switchTool(tool.id)"
      >
        <span class="tool-icon">{{ tool.icon }}</span>
        <span class="tool-name">{{ tool.name }}</span>
      </button>

      <div v-if="store.filteredTools.length === 0" class="empty-tools">
        未找到匹配工具
      </div>
    </nav>

    <footer class="sidebar-footer">
      <div class="footer-meta">v0.1.0 · Tauri 2</div>
    </footer>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 220px;
  background: linear-gradient(180deg, var(--bg-elev-1) 0%, var(--bg-base) 100%);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  position: relative;
}

.sidebar::after {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  width: 1px;
  height: 100%;
  background: linear-gradient(
    180deg,
    transparent,
    rgba(99, 102, 241, 0.2),
    transparent
  );
  pointer-events: none;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 16px 14px;
  border-bottom: 1px solid var(--border);
}

.brand-logo {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
}

.brand-logo svg {
  width: 100%;
  height: 100%;
  display: block;
}

.brand-text {
  min-width: 0;
}

.brand-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-bright);
  letter-spacing: -0.01em;
}

.brand-sub {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 1px;
}

.search-box {
  position: relative;
  padding: 12px 12px 8px;
}

.search-icon {
  position: absolute;
  left: 22px;
  top: 50%;
  transform: translateY(-50%);
  width: 14px;
  height: 14px;
  color: var(--text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 7px 10px 7px 30px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elev-2);
  color: var(--text);
  font-size: 12.5px;
  outline: none;
  transition: all var(--transition);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-input:focus {
  border-color: var(--border-focus);
  background: var(--bg-elev-3);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.12);
}

.section-label {
  padding: 8px 16px 6px;
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-muted);
}

.tool-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tool-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: all var(--transition);
}

.tool-item:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.tool-item.active {
  background: var(--bg-active);
  color: var(--text-bright);
}

.tool-item.active::before {
  content: "";
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 16px;
  background: var(--accent);
  border-radius: 0 2px 2px 0;
  box-shadow: 0 0 8px var(--accent-glow);
}

.tool-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.tool-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.empty-tools {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.sidebar-footer {
  padding: 10px 16px 14px;
  border-top: 1px solid var(--border);
}

.footer-meta {
  font-size: 10.5px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.04em;
}
</style>
