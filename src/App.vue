<script setup lang="ts">
import { onMounted } from "vue";
import Sidebar from "./layout/Sidebar.vue";
import ToolContainer from "./layout/ToolContainer.vue";
import Toast from "./components/Toast.vue";
import { useThemeStore } from "./stores/theme";

const theme = useThemeStore();
onMounted(() => theme.applyTheme());
</script>

<template>
  <div class="app">
    <Sidebar />
    <ToolContainer />
    <Toast />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --bg-base: #0d1017;
  --bg-elev-1: #151a24;
  --bg-elev-2: #1b2230;
  --bg-elev-3: #242c3c;
  --bg-hover: rgba(255, 255, 255, 0.04);
  --bg-active: rgba(99, 102, 241, 0.14);
  --bg-header: rgba(13, 16, 23, 0.7);
  --bg-toolbar: rgba(13, 16, 23, 0.5);
  --bg-glass: rgba(21, 26, 36, 0.65);

  --border: rgba(255, 255, 255, 0.07);
  --border-strong: rgba(255, 255, 255, 0.13);
  --border-focus: rgba(99, 102, 241, 0.55);

  --text: #e5e7eb;
  --text-dim: #b3b8c2;
  --text-muted: #8b919c;
  --text-bright: #fafafa;

  --accent: #6366f1;
  --accent-hover: #818cf8;
  --accent-glow: rgba(99, 102, 241, 0.35);

  --success: #10b981;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: #3b82f6;

  --type-text: #60a5fa;
  --type-image: #34d399;
  --type-files: #fbbf24;

  --radius-sm: 6px;
  --radius: 10px;
  --radius-lg: 14px;
  --radius-xl: 18px;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.55);

  --transition: 150ms cubic-bezier(0.4, 0, 0.2, 1);

  color-scheme: dark;
}

:root[data-theme="light"] {
  --bg-base: #f7f8fa;
  --bg-elev-1: #ffffff;
  --bg-elev-2: #f1f3f7;
  --bg-elev-3: #e6eaf0;
  --bg-hover: rgba(0, 0, 0, 0.04);
  --bg-active: rgba(99, 102, 241, 0.08);
  --bg-header: rgba(255, 255, 255, 0.78);
  --bg-toolbar: rgba(247, 248, 250, 0.6);
  --bg-glass: rgba(255, 255, 255, 0.7);

  --border: rgba(0, 0, 0, 0.08);
  --border-strong: rgba(0, 0, 0, 0.12);
  --border-focus: rgba(99, 102, 241, 0.5);

  --text: #1f2937;
  --text-dim: #4b5563;
  --text-muted: #9ca3af;
  --text-bright: #0f172a;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.06);
  --shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.12);

  color-scheme: light;
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
  transition: background-color 0.25s ease, color 0.25s ease;
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
  background: rgba(128, 128, 128, 0.2);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(128, 128, 128, 0.4);
}

.app {
  display: flex;
  height: 100vh;
  width: 100vw;
  background:
    radial-gradient(ellipse at top left, var(--accent-glow), transparent 55%),
    radial-gradient(ellipse at bottom right, var(--accent-glow), transparent 60%),
    var(--bg-base);
  transition: background-color 0.25s ease;
}

:root[data-theme="light"] .app {
  background:
    radial-gradient(ellipse at top left, rgba(99, 102, 241, 0.08), transparent 55%),
    radial-gradient(ellipse at bottom right, rgba(168, 85, 247, 0.06), transparent 60%),
    var(--bg-base);
}
</style>
