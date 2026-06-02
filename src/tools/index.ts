import type { Component } from "vue";
import type { ToolDef } from "../types";
import ClipboardManager from "./ClipboardManager.vue";
import Settings from "./Settings.vue";

export interface ToolRegistration extends ToolDef {
  component: Component
}

export const tools: ToolRegistration[] = [
  {
    id: "clipboard",
    name: "剪贴板管理",
    icon: "📋",
    desc: "浏览和管理剪贴板历史记录",
    keywords: ["复制", "历史", "粘贴", "剪贴板", "clipboard"],
    component: ClipboardManager,
  },
  {
    id: "settings",
    name: "设置",
    icon: "⚙️",
    desc: "主题、外观与个性化配置",
    keywords: ["设置", "主题", "外观", "偏好", "settings", "theme", "preferences"],
    component: Settings,
  },
]

export function getTool(id: string): ToolRegistration | undefined {
  return tools.find((t) => t.id === id)
}
