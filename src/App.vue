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
  --bg: #1a1a2e;
  --bg-hover: #16213e;
  --bg-selected: #0f3460;
  --text: #e0e0e0;
  --text-dim: #8892a4;
  --text-bright: #ffffff;
  --accent: #e94560;
  --border: #2a2a4a;
  --radius: 8px;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
  background: var(--bg);
  color: var(--text);
  overflow: hidden;
  user-select: none;
}

.app {
  display: flex;
  height: 100vh;
  width: 100vw;
}
</style>
