<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  danger?: boolean;
  disabled?: boolean;
  separator?: false;
}

export interface ContextMenuSeparator {
  id: string;
  separator: true;
}

export type ContextMenuEntry = ContextMenuItem | ContextMenuSeparator;

const props = defineProps<{
  x: number;
  y: number;
  items: ContextMenuEntry[];
}>();

const emit = defineEmits<{
  select: [id: string];
  close: [];
}>();

const menuRef = ref<HTMLDivElement | null>(null);

function onWindowClick(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit("close");
  }
}

function onWindowKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => {
  window.addEventListener("mousedown", onWindowClick, true);
  window.addEventListener("keydown", onWindowKeydown, true);
});

onUnmounted(() => {
  window.removeEventListener("mousedown", onWindowClick, true);
  window.removeEventListener("keydown", onWindowKeydown, true);
});

function adjustedPos(): { left: string; top: string } {
  const margin = 4;
  const w = 180;
  const h = props.items.length * 32 + 12;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const left = props.x + w + margin > vw ? Math.max(margin, props.x - w) : props.x;
  const top = props.y + h + margin > vh ? Math.max(margin, props.y - h) : props.y;
  return { left: `${left}px`, top: `${top}px` };
}

function onItemClick(item: ContextMenuItem) {
  if (item.disabled) return;
  emit("select", item.id);
  emit("close");
}
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      class="ctx-menu"
      :style="adjustedPos()"
      @click.stop
      @contextmenu.prevent
    >
      <template v-for="item in props.items" :key="item.id">
        <div v-if="'separator' in item && item.separator" class="ctx-sep" />
        <button
          v-else
          type="button"
          class="ctx-item"
          :class="{ danger: item.danger, disabled: item.disabled }"
          :disabled="item.disabled"
          @click="onItemClick(item)"
        >
          <span class="ctx-label">{{ item.label }}</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.ctx-menu {
  position: fixed;
  z-index: 2000;
  min-width: 180px;
  padding: 4px;
  background: var(--bg-elev-1);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  box-shadow: var(--shadow-lg);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  animation: ctx-pop 120ms ease-out;
}

@keyframes ctx-pop {
  from {
    opacity: 0;
    transform: scale(0.96);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.ctx-item {
  display: flex;
  width: 100%;
  align-items: center;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  font-size: 12.5px;
  text-align: left;
  cursor: pointer;
  transition: background var(--transition);
}

.ctx-item:hover:not(.disabled) {
  background: var(--bg-active);
  color: var(--text-bright);
}

.ctx-item.danger {
  color: var(--danger);
}

.ctx-item.danger:hover:not(.disabled) {
  background: rgba(239, 68, 68, 0.12);
}

.ctx-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.ctx-sep {
  height: 1px;
  margin: 4px 6px;
  background: var(--border);
}
</style>
