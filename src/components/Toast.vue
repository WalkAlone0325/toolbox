<script setup lang="ts">
import { useToastStore } from "../stores/toast";

const toast = useToastStore();
</script>

<template>
  <Teleport to="body">
    <TransitionGroup name="toast" tag="div" class="toast-container">
      <div
        v-for="item in toast.items"
        :key="item.id"
        class="toast"
        :class="`kind-${item.kind}`"
        @click="toast.dismiss(item.id)"
      >
        <svg
          v-if="item.kind === 'success'"
          class="toast-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
        >
          <path d="m5 12 5 5 9-11" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg
          v-else-if="item.kind === 'info'"
          class="toast-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
        >
          <circle cx="12" cy="12" r="9" />
          <path d="M12 8.5h.01M11 12h1v5h1" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg
          v-else-if="item.kind === 'warning'"
          class="toast-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
        >
          <path d="M12 3 2 21h20L12 3ZM12 10v5M12 18.5h.01" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg
          v-else
          class="toast-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
        >
          <circle cx="12" cy="12" r="9" />
          <path d="m9 9 6 6M15 9l-6 6" stroke-linecap="round" />
        </svg>
        <span class="toast-msg">{{ item.message }}</span>
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 9999;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px 10px 14px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  border: 1px solid var(--border-strong);
  color: var(--text-bright);
  font-size: 13px;
  font-weight: 500;
  box-shadow: var(--shadow-lg);
  backdrop-filter: blur(14px) saturate(180%);
  -webkit-backdrop-filter: blur(14px) saturate(180%);
  cursor: pointer;
  pointer-events: auto;
  min-width: 180px;
  max-width: 360px;
}

.toast.kind-success {
  border-color: rgba(16, 185, 129, 0.4);
}
.toast.kind-success .toast-icon {
  color: var(--success);
}

.toast.kind-info {
  border-color: rgba(99, 102, 241, 0.4);
}
.toast.kind-info .toast-icon {
  color: var(--accent);
}

.toast.kind-warning {
  border-color: rgba(245, 158, 11, 0.4);
}
.toast.kind-warning .toast-icon {
  color: var(--warning);
}

.toast.kind-danger {
  border-color: rgba(239, 68, 68, 0.4);
}
.toast.kind-danger .toast-icon {
  color: var(--danger);
}

.toast-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.toast-msg {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}
.toast-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}
.toast-enter-active,
.toast-leave-active {
  transition: all 220ms cubic-bezier(0.4, 0, 0.2, 1);
}
.toast-move {
  transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
