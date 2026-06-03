import { defineStore } from "pinia";
import { ref } from "vue";

export type ToastKind = "success" | "info" | "warning" | "danger";

export interface ToastItem {
  id: number;
  kind: ToastKind;
  message: string;
}

let counter = 0;

export const useToastStore = defineStore("toast", () => {
  const items = ref<ToastItem[]>([]);
  let timers = new Map<number, ReturnType<typeof setTimeout>>();

  function show(message: string, kind: ToastKind = "info", duration = 1800) {
    const id = ++counter;
    items.value.push({ id, kind, message });
    const timer = setTimeout(() => dismiss(id), duration);
    timers.set(id, timer);
  }

  function dismiss(id: number) {
    items.value = items.value.filter((t) => t.id !== id);
    const t = timers.get(id);
    if (t) {
      clearTimeout(t);
      timers.delete(id);
    }
  }

  function success(msg: string) {
    show(msg, "success");
  }
  function info(msg: string) {
    show(msg, "info");
  }
  function warning(msg: string) {
    show(msg, "warning");
  }
  function danger(msg: string) {
    show(msg, "danger", 2500);
  }

  function preview(message: string, kind: ToastKind = "success") {
    const text = message.replace(/\s+/g, " ").trim();
    const snippet = text.length > 24 ? `${text.slice(0, 24)}…` : text;
    show(snippet ? `已复制：${snippet}` : "已复制", kind);
  }

  return { items, show, dismiss, success, info, warning, danger, preview };
});
