<script setup lang="ts">
import { computed } from "vue";
import { useThemeStore, THEME_PRESETS, type ThemeMode } from "../stores/theme";

const theme = useThemeStore();

const modes: { id: ThemeMode; label: string; desc: string; icon: string }[] = [
  { id: "system", label: "跟随系统", desc: "自动匹配 macOS 外观", icon: "auto" },
  { id: "dark", label: "深色", desc: "始终使用深色主题", icon: "moon" },
  { id: "light", label: "浅色", desc: "始终使用浅色主题", icon: "sun" },
];

const accentPresets = [
  "#6366f1",
  "#0ea5e9",
  "#10b981",
  "#f59e0b",
  "#ef4444",
  "#ec4899",
  "#a855f7",
  "#14b8a6",
];

const systemStatus = computed(() =>
  theme.systemDark ? "系统当前：深色" : "系统当前：浅色"
);
const effectiveLabel = computed(() => (theme.isDark ? "深色" : "浅色"));
</script>

<template>
  <div class="settings-tool">
    <section class="block">
      <header class="block-header">
        <h2 class="block-title">外观模式</h2>
        <p class="block-desc">控制应用整体使用深色或浅色主题</p>
      </header>

      <div class="mode-grid">
        <button
          v-for="m in modes"
          :key="m.id"
          class="mode-card"
          :class="{ active: theme.mode === m.id }"
          @click="theme.setMode(m.id)"
        >
          <div class="mode-icon">
            <svg v-if="m.icon === 'auto'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="9" />
              <path d="M12 3a9 9 0 0 0 0 18Z" fill="currentColor" stroke="none" />
            </svg>
            <svg v-else-if="m.icon === 'moon'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79Z" stroke-linejoin="round" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="4" />
              <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" stroke-linecap="round" />
            </svg>
          </div>
          <div class="mode-text">
            <div class="mode-label">{{ m.label }}</div>
            <div class="mode-desc">{{ m.desc }}</div>
          </div>
          <div v-if="theme.mode === m.id" class="check">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <path d="m5 12 5 5 9-11" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
        </button>
      </div>

      <div class="status-bar">
        <span class="status-dot" :class="{ dark: theme.isDark }"></span>
        <span>{{ systemStatus }} · 当前生效：<strong>{{ effectiveLabel }}</strong></span>
      </div>
    </section>

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">主题预设</h2>
        <p class="block-desc">选择喜欢的配色风格（仅在深色模式下完整生效）</p>
      </header>

      <div class="preset-grid">
        <button
          v-for="p in THEME_PRESETS"
          :key="p.id"
          class="preset-card"
          :class="{ active: theme.presetId === p.id }"
          @click="theme.setPreset(p.id)"
        >
          <div class="preset-swatch">
            <div
              v-for="(c, i) in p.swatch"
              :key="i"
              class="swatch-color"
              :style="{ background: c }"
            ></div>
          </div>
          <div class="preset-info">
            <div class="preset-name">{{ p.name }}</div>
            <div class="preset-desc">{{ p.desc }}</div>
          </div>
          <div v-if="theme.presetId === p.id" class="check">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <path d="m5 12 5 5 9-11" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
        </button>
      </div>
    </section>

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">自定义主色</h2>
        <p class="block-desc">覆盖预设的强调色，应用于按钮、聚焦、选中态</p>
      </header>

      <div class="accent-row">
        <label class="accent-toggle">
          <input
            type="checkbox"
            :checked="theme.customAccent !== null"
            @change="
              theme.setCustomAccent(
                ($event.target as HTMLInputElement).checked ? accentPresets[0] : null
              )
            "
          />
          <span>启用自定义主色</span>
        </label>

        <div class="accent-options" :class="{ disabled: theme.customAccent === null }">
          <button
            v-for="c in accentPresets"
            :key="c"
            class="accent-dot"
            :class="{ active: theme.customAccent === c }"
            :style="{ background: c }"
            :title="c"
            @click="theme.setCustomAccent(c)"
          ></button>
          <label class="accent-custom" title="自定义颜色">
            <input
              type="color"
              :value="theme.customAccent ?? accentPresets[0]"
              :disabled="theme.customAccent === null"
              @input="
                theme.setCustomAccent(($event.target as HTMLInputElement).value)
              "
            />
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v4M12 18v4M2 12h4M18 12h4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" stroke-linecap="round" />
            </svg>
          </label>
        </div>
      </div>
    </section>

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">重置</h2>
        <p class="block-desc">恢复默认主题配置</p>
      </header>
      <button class="reset-btn" @click="theme.reset()">
        恢复默认设置
      </button>
    </section>

    <footer class="settings-footer">
      <span>Toolbox v0.1.0 · 配置存储于本地浏览器</span>
    </footer>
  </div>
</template>

<style scoped>
.settings-tool {
  padding: 24px 28px 40px;
  overflow-y: auto;
  height: 100%;
  max-width: 880px;
  margin: 0 auto;
}

.block {
  background: var(--bg-elev-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 20px 22px;
  margin-bottom: 16px;
}

.block-header {
  margin-bottom: 16px;
}

.block-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-bright);
  letter-spacing: -0.01em;
}

.block-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 3px;
}

.mode-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.mode-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 38px 14px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: all var(--transition);
  min-height: 64px;
}

.mode-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-elev-3);
}

.mode-card.active {
  border-color: var(--accent);
  background: var(--bg-active);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.mode-icon {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  background: var(--bg-elev-3);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  flex-shrink: 0;
}

.mode-card.active .mode-icon {
  background: var(--accent);
  color: white;
}

.mode-icon svg {
  width: 18px;
  height: 18px;
}

.mode-text {
  flex: 1;
  min-width: 0;
}

.mode-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-bright);
}

.mode-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  margin-top: 2px;
}

.mode-card .check,
.preset-card .check {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 16px;
  height: 16px;
  color: var(--accent);
  background: var(--bg-elev-1);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 0 0 2px var(--bg-elev-2);
}

.mode-card.active .check,
.preset-card.active .check {
  background: var(--accent);
  color: white;
  box-shadow: 0 0 0 2px var(--bg-elev-1), 0 0 8px var(--accent-glow);
}

.check svg {
  width: 12px;
  height: 12px;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 14px;
  padding: 10px 12px;
  background: var(--bg-elev-2);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--text-dim);
}

.status-bar strong {
  color: var(--text-bright);
  font-weight: 600;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--warning);
  box-shadow: 0 0 6px currentColor;
}

.status-dot.dark {
  background: var(--accent);
}

.preset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}

.preset-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 36px 12px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  cursor: pointer;
  text-align: left;
  transition: all var(--transition);
}

.preset-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-elev-3);
  transform: translateY(-1px);
}

.preset-card.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.preset-swatch {
  display: flex;
  width: 42px;
  height: 42px;
  border-radius: var(--radius-sm);
  overflow: hidden;
  flex-shrink: 0;
  border: 1px solid var(--border-strong);
}

.swatch-color {
  flex: 1;
}

.preset-info {
  flex: 1;
  min-width: 0;
}

.preset-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-bright);
}

.preset-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  margin-top: 2px;
}

.accent-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}

.accent-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
}

.accent-toggle input {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}

.accent-options {
  display: flex;
  align-items: center;
  gap: 8px;
}

.accent-options.disabled {
  opacity: 0.4;
  pointer-events: none;
}

.accent-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: transform var(--transition);
  padding: 0;
}

.accent-dot:hover {
  transform: scale(1.12);
}

.accent-dot.active {
  border-color: white;
  box-shadow: 0 0 0 2px var(--accent), 0 4px 10px rgba(0, 0, 0, 0.3);
}

.accent-custom {
  position: relative;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: conic-gradient(
    from 0deg,
    #ef4444,
    #f59e0b,
    #10b981,
    #3b82f6,
    #6366f1,
    #ec4899,
    #ef4444
  );
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--bg-elev-1);
}

.accent-custom input[type="color"] {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.accent-custom svg {
  width: 12px;
  height: 12px;
  color: white;
  pointer-events: none;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.4));
}

.reset-btn {
  padding: 8px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elev-2);
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition);
}

.reset-btn:hover {
  border-color: var(--danger);
  color: var(--danger);
  background: rgba(239, 68, 68, 0.08);
}

.settings-footer {
  text-align: center;
  font-size: 11.5px;
  color: var(--text-muted);
  padding: 12px 0;
}
</style>
