<script setup lang="ts">
import Sidebar from "./layout/Sidebar.vue";
import ToolContainer from "./layout/ToolContainer.vue";
</script>

<template>
  <div class="app">
    <Sidebar />
    <ToolContainer />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --bg-base: #0b0d12;
  --bg-elev-1: #11141b;
  --bg-elev-2: #161a23;
  --bg-elev-3: #1d2230;
  --bg-hover: rgba(255, 255, 255, 0.04);
  --bg-active: rgba(99, 102, 241, 0.12);

  --border: rgba(255, 255, 255, 0.06);
  --border-strong: rgba(255, 255, 255, 0.1);
  --border-focus: rgba(99, 102, 241, 0.5);

  --text: #e6e9ef;
  --text-dim: #8b93a7;
  --text-muted: #5a6275;
  --text-bright: #ffffff;

  --accent: #6366f1;
  --accent-hover: #7c7ff5;
  --accent-glow: rgba(99, 102, 241, 0.4);

  --success: #10b981;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: #3b82f6;

  --type-text: #3b82f6;
  --type-image: #10b981;
  --type-files: #f59e0b;

  --radius-sm: 6px;
  --radius: 10px;
  --radius-lg: 14px;
  --radius-xl: 18px;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.5);

  --transition: 150ms cubic-bezier(0.4, 0, 0.2, 1);
}

html,
body,
#app {
  height: 100%;
  width: 100%;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Inter", "Segoe UI",
    "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", Roboto, Helvetica,
    Arial, sans-serif;
  background: var(--bg-base);
  color: var(--text);
  overflow: hidden;
  user-select: none;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  font-size: 14px;
  line-height: 1.5;
}

button {
  font-family: inherit;
}

input {
  font-family: inherit;
}

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.12);
}

.app {
  display: flex;
  height: 100vh;
  width: 100vw;
  background: radial-gradient(
      ellipse at top left,
      rgba(99, 102, 241, 0.06),
      transparent 50%
    ),
    radial-gradient(
      ellipse at bottom right,
      rgba(16, 185, 129, 0.04),
      transparent 50%
    ),
    var(--bg-base);
}
</style>
