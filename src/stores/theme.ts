import { defineStore } from "pinia";
import { ref, computed, watch, onMounted, onUnmounted } from "vue";

export type ThemeMode = "system" | "dark" | "light";

export interface ThemePreset {
  id: string;
  name: string;
  desc: string;
  swatch: string[];
  vars: Record<string, string>;
}

export const THEME_PRESETS: ThemePreset[] = [
  {
    id: "midnight",
    name: "午夜",
    desc: "靛紫渐变 · 默认",
    swatch: ["#818cf8", "#a855f7", "#ec4899"],
    vars: {
      "--bg-base": "#0b0d12",
      "--bg-elev-1": "#11141b",
      "--bg-elev-2": "#161a23",
      "--bg-elev-3": "#1d2230",
      "--accent": "#6366f1",
      "--accent-hover": "#7c7ff5",
      "--accent-glow": "rgba(99, 102, 241, 0.4)",
      "--border-focus": "rgba(99, 102, 241, 0.5)",
    },
  },
  {
    id: "ocean",
    name: "深海",
    desc: "蓝色 · 冷静专注",
    swatch: ["#38bdf8", "#0ea5e9", "#0369a1"],
    vars: {
      "--bg-base": "#0a1421",
      "--bg-elev-1": "#0f1b2d",
      "--bg-elev-2": "#142236",
      "--bg-elev-3": "#1b2d45",
      "--accent": "#0ea5e9",
      "--accent-hover": "#38bdf8",
      "--accent-glow": "rgba(14, 165, 233, 0.4)",
      "--border-focus": "rgba(14, 165, 233, 0.5)",
    },
  },
  {
    id: "forest",
    name: "森林",
    desc: "绿色 · 自然护眼",
    swatch: ["#34d399", "#10b981", "#047857"],
    vars: {
      "--bg-base": "#0a1410",
      "--bg-elev-1": "#0f1c17",
      "--bg-elev-2": "#14241d",
      "--bg-elev-3": "#1b2f26",
      "--accent": "#10b981",
      "--accent-hover": "#34d399",
      "--accent-glow": "rgba(16, 185, 129, 0.4)",
      "--border-focus": "rgba(16, 185, 129, 0.5)",
    },
  },
  {
    id: "sunset",
    name: "日落",
    desc: "橙红 · 温暖活力",
    swatch: ["#fb923c", "#f97316", "#ea580c"],
    vars: {
      "--bg-base": "#14100c",
      "--bg-elev-1": "#1c1712",
      "--bg-elev-2": "#241d17",
      "--bg-elev-3": "#2f261d",
      "--accent": "#f97316",
      "--accent-hover": "#fb923c",
      "--accent-glow": "rgba(249, 115, 22, 0.4)",
      "--border-focus": "rgba(249, 115, 22, 0.5)",
    },
  },
  {
    id: "rose",
    name: "玫瑰",
    desc: "粉红 · 柔和优雅",
    swatch: ["#fb7185", "#f43f5e", "#be123c"],
    vars: {
      "--bg-base": "#150e12",
      "--bg-elev-1": "#1d141a",
      "--bg-elev-2": "#251821",
      "--bg-elev-3": "#301f2b",
      "--accent": "#f43f5e",
      "--accent-hover": "#fb7185",
      "--accent-glow": "rgba(244, 63, 94, 0.4)",
      "--border-focus": "rgba(244, 63, 94, 0.5)",
    },
  },
  {
    id: "slate",
    name: "石板",
    desc: "中性灰 · 极简",
    swatch: ["#94a3b8", "#64748b", "#334155"],
    vars: {
      "--bg-base": "#0c1015",
      "--bg-elev-1": "#131820",
      "--bg-elev-2": "#1a212b",
      "--bg-elev-3": "#232c38",
      "--accent": "#64748b",
      "--accent-hover": "#94a3b8",
      "--accent-glow": "rgba(100, 116, 139, 0.4)",
      "--border-focus": "rgba(100, 116, 139, 0.5)",
    },
  },
];

const LIGHT_THEME = {
  vars: {
    "--bg-base": "#f7f8fa",
    "--bg-elev-1": "#ffffff",
    "--bg-elev-2": "#f1f3f7",
    "--bg-elev-3": "#e6eaf0",
    "--bg-hover": "rgba(0, 0, 0, 0.04)",
    "--bg-active": "rgba(99, 102, 241, 0.08)",
    "--border": "rgba(0, 0, 0, 0.08)",
    "--border-strong": "rgba(0, 0, 0, 0.12)",
    "--text": "#1f2937",
    "--text-dim": "#4b5563",
    "--text-muted": "#9ca3af",
    "--text-bright": "#0f172a",
  },
};

const STORAGE_KEY = "toolbox.theme";

interface StoredConfig {
  mode: ThemeMode;
  presetId: string;
  customAccent: string | null;
  customBgBase: string | null;
}

function loadConfig(): StoredConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultConfig();
    const parsed = JSON.parse(raw) as Partial<StoredConfig>;
    return {
      mode: parsed.mode ?? "system",
      presetId: parsed.presetId ?? "midnight",
      customAccent: parsed.customAccent ?? null,
      customBgBase: parsed.customBgBase ?? null,
    };
  } catch {
    return defaultConfig();
  }
}

function defaultConfig(): StoredConfig {
  return { mode: "system", presetId: "midnight", customAccent: null, customBgBase: null };
}

function isDarkMode(mode: ThemeMode, systemDark: boolean): boolean {
  if (mode === "system") return systemDark;
  return mode === "dark";
}

function hexToRgb(hex: string): [number, number, number] | null {
  const m = hex.replace("#", "");
  if (m.length !== 6) return null;
  const r = parseInt(m.slice(0, 2), 16);
  const g = parseInt(m.slice(2, 4), 16);
  const b = parseInt(m.slice(4, 6), 16);
  if ([r, g, b].some((n) => Number.isNaN(n))) return null;
  return [r, g, b];
}

function withAlpha(hex: string, alpha: number): string {
  const rgb = hexToRgb(hex);
  if (!rgb) return hex;
  return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}

export const useThemeStore = defineStore("theme", () => {
  const initial = loadConfig();
  const mode = ref<ThemeMode>(initial.mode);
  const presetId = ref<string>(initial.presetId);
  const customAccent = ref<string | null>(initial.customAccent);
  const customBgBase = ref<string | null>(initial.customBgBase);
  const systemDark = ref(
    typeof window !== "undefined"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
      : true
  );

  let mediaQuery: MediaQueryList | null = null;
  const onMediaChange = (e: MediaQueryListEvent) => {
    systemDark.value = e.matches;
  };

  onMounted(() => {
    if (typeof window === "undefined") return;
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    systemDark.value = mediaQuery.matches;
    mediaQuery.addEventListener("change", onMediaChange);
  });

  onUnmounted(() => {
    mediaQuery?.removeEventListener("change", onMediaChange);
  });

  const isDark = computed(() => isDarkMode(mode.value, systemDark.value));
  const activePreset = computed(
    () => THEME_PRESETS.find((p) => p.id === presetId.value) ?? THEME_PRESETS[0]
  );

  function applyTheme() {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.dataset.theme = isDark.value ? "dark" : "light";
    root.dataset.preset = presetId.value;

    const target = activePreset.value;
    for (const [k, v] of Object.entries(target.vars)) {
      root.style.setProperty(k, v);
    }

    if (customAccent.value) {
      const accent = customAccent.value;
      root.style.setProperty("--accent", accent);
      const rgb = hexToRgb(accent);
      if (rgb) {
        const hover = `rgb(${Math.min(rgb[0] + 30, 255)}, ${Math.min(rgb[1] + 30, 255)}, ${Math.min(rgb[2] + 30, 255)})`;
        root.style.setProperty("--accent-hover", hover);
        root.style.setProperty("--accent-glow", withAlpha(accent, 0.4));
        root.style.setProperty("--border-focus", withAlpha(accent, 0.5));
        root.style.setProperty("--bg-active", withAlpha(accent, isDark.value ? 0.12 : 0.08));
      }
    } else {
      root.style.removeProperty("--bg-active");
    }

    if (customBgBase.value) {
      root.style.setProperty("--bg-base", customBgBase.value);
    }

    if (!isDark.value) {
      for (const [k, v] of Object.entries(LIGHT_THEME.vars)) {
        root.style.setProperty(k, v);
      }
      if (customAccent.value) {
        const accent = customAccent.value;
        root.style.setProperty("--accent", accent);
        const rgb = hexToRgb(accent);
        if (rgb) {
          root.style.setProperty("--accent-hover", `rgb(${Math.min(rgb[0] + 30, 255)}, ${Math.min(rgb[1] + 30, 255)}, ${Math.min(rgb[2] + 30, 255)})`);
          root.style.setProperty("--accent-glow", withAlpha(accent, 0.3));
          root.style.setProperty("--border-focus", withAlpha(accent, 0.5));
          root.style.setProperty("--bg-active", withAlpha(accent, 0.08));
        }
      }
    }
  }

  function setMode(m: ThemeMode) {
    mode.value = m;
  }

  function setPreset(id: string) {
    presetId.value = id;
  }

  function setCustomAccent(color: string | null) {
    customAccent.value = color;
  }

  function setCustomBg(color: string | null) {
    customBgBase.value = color;
  }

  function reset() {
    mode.value = "system";
    presetId.value = "midnight";
    customAccent.value = null;
    customBgBase.value = null;
  }

  watch(
    [mode, presetId, customAccent, customBgBase, systemDark, isDark],
    () => {
      const cfg: StoredConfig = {
        mode: mode.value,
        presetId: presetId.value,
        customAccent: customAccent.value,
        customBgBase: customBgBase.value,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(cfg));
      applyTheme();
    },
    { deep: true, immediate: true }
  );

  return {
    mode,
    presetId,
    customAccent,
    customBgBase,
    systemDark,
    isDark,
    activePreset,
    setMode,
    setPreset,
    setCustomAccent,
    setCustomBg,
    reset,
    applyTheme,
  };
});
