<script setup lang="ts">
import { computed, ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useThemeStore, THEME_PRESETS, type ThemeMode } from "../stores/theme";
import { useToastStore } from "../stores/toast";
import { useDisplayStore, type FontSize } from "../stores/display";

const theme = useThemeStore();
const toast = useToastStore();
const display = useDisplayStore();

interface AppSettings {
  auto_hide_on_blur: boolean;
  max_age_days: number | null;
  max_count: number | null;
  max_bytes: number | null;
  ignore_apps: string[];
  cleanup_interval_secs: number;
  llm_provider: string | null;
  llm_api_key: string | null;
  llm_base_url: string | null;
  llm_model: string | null;
}

const settings = ref<AppSettings>({
  auto_hide_on_blur: false,
  max_age_days: null,
  max_count: null,
  max_bytes: null,
  ignore_apps: [],
  cleanup_interval_secs: 3600,
  llm_provider: null,
  llm_api_key: null,
  llm_base_url: null,
  llm_model: null,
});

const autoHideOnBlur = ref(false);
const maxAgeDays = ref<number | null>(null);
const maxCount = ref<number | null>(null);
const maxBytesMB = ref<number | null>(null);
const ignoreApps = ref<string[]>([]);
const newIgnoreApp = ref("");

const llmProvider = ref<string>("openai");
const llmApiKey = ref<string>("");
const llmBaseUrl = ref<string>("");
const llmModel = ref<string>("");
const testing = ref(false);
const testResult = ref<{ ok: boolean; message: string } | null>(null);

const providerOptions = [
  { id: "openai", label: "OpenAI", default_base: "https://api.openai.com/v1", default_model: "gpt-4o-mini" },
  { id: "anthropic", label: "Anthropic Claude", default_base: "https://api.anthropic.com", default_model: "claude-haiku-4-5-20251001" },
  { id: "ollama", label: "Ollama (本地)", default_base: "http://localhost:11434", default_model: "qwen2.5:7b" },
];

const currentProvider = computed(() =>
  providerOptions.find((p) => p.id === llmProvider.value) ?? providerOptions[0]
);

function onProviderChange() {
  if (!llmBaseUrl.value) llmBaseUrl.value = currentProvider.value.default_base;
  if (!llmModel.value) llmModel.value = currentProvider.value.default_model;
}

watch(llmProvider, onProviderChange);

watch(autoHideOnBlur, (v) => {
  localStorage.setItem("sparkbox.autoHideOnBlur", v ? "1" : "0");
  window.dispatchEvent(new CustomEvent("sparkbox-settings-changed"));
});

async function loadSettings() {
  try {
    const s = await invoke<AppSettings>("get_settings");
    settings.value = s;
    autoHideOnBlur.value = s.auto_hide_on_blur || localStorage.getItem("sparkbox.autoHideOnBlur") === "1";
    maxAgeDays.value = s.max_age_days;
    maxCount.value = s.max_count;
    maxBytesMB.value = s.max_bytes ? Math.round(s.max_bytes / (1024 * 1024)) : null;
    ignoreApps.value = [...s.ignore_apps];
    llmProvider.value = s.llm_provider ?? "openai";
    llmApiKey.value = s.llm_api_key ?? "";
    llmBaseUrl.value = s.llm_base_url ?? currentProvider.value.default_base;
    llmModel.value = s.llm_model ?? currentProvider.value.default_model;
  } catch (e) {
    autoHideOnBlur.value = localStorage.getItem("sparkbox.autoHideOnBlur") === "1";
  }
}

async function saveAll() {
  try {
    await invoke("save_settings", {
      settings: {
        auto_hide_on_blur: autoHideOnBlur.value,
        max_age_days: maxAgeDays.value && maxAgeDays.value > 0 ? maxAgeDays.value : null,
        max_count: maxCount.value && maxCount.value > 0 ? maxCount.value : null,
        max_bytes: maxBytesMB.value && maxBytesMB.value > 0 ? maxBytesMB.value * 1024 * 1024 : null,
        ignore_apps: ignoreApps.value,
        cleanup_interval_secs: 3600,
        llm_provider: llmProvider.value || null,
        llm_api_key: llmApiKey.value || null,
        llm_base_url: llmBaseUrl.value || null,
        llm_model: llmModel.value || null,
      },
    });
    toast.success("设置已保存");
  } catch {
    toast.danger("保存失败");
  }
}

async function testConnection() {
  if (testing.value) return;
  await saveAll();
  testing.value = true;
  testResult.value = null;
  try {
    const reply = await invoke<string>("test_llm_connection");
    testResult.value = { ok: true, message: `连接成功：${reply || "(空回复)"}` };
  } catch (e) {
    testResult.value = { ok: false, message: `失败：${(e as Error).message || e}` };
  } finally {
    testing.value = false;
  }
}

async function cleanupNow() {
  try {
    const n = await invoke<number>("cleanup_now", {
      maxAgeDays: maxAgeDays.value && maxAgeDays.value > 0 ? maxAgeDays.value : null,
      maxCount: maxCount.value && maxCount.value > 0 ? maxCount.value : null,
      maxBytes: maxBytesMB.value && maxBytesMB.value > 0 ? maxBytesMB.value * 1024 * 1024 : null,
    });
    toast.success(`已清理 ${n} 条`);
  } catch {
    toast.danger("清理失败");
  }
}

function addIgnoreApp() {
  const name = newIgnoreApp.value.trim();
  if (!name) return;
  if (!ignoreApps.value.includes(name)) {
    ignoreApps.value.push(name);
  }
  newIgnoreApp.value = "";
}

function removeIgnoreApp(name: string) {
  ignoreApps.value = ignoreApps.value.filter((x) => x !== name);
}

onMounted(loadSettings);

const modes: { id: ThemeMode; label: string; desc: string; icon: string }[] = [
  { id: "system", label: "跟随系统", desc: "自动匹配系统外观", icon: "auto" },
  { id: "dark", label: "深色", desc: "始终使用深色", icon: "moon" },
  { id: "light", label: "浅色", desc: "始终使用浅色", icon: "sun" },
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

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">行为</h2>
        <p class="block-desc">控制窗口与剪贴板的交互行为</p>
      </header>

      <label class="row">
        <div class="row-text">
          <div class="row-label">窗口失焦自动隐藏</div>
          <div class="row-desc">点击其他应用时窗口自动隐藏（类 PasteBot 风格）</div>
        </div>
        <input v-model="autoHideOnBlur" type="checkbox" class="toggle" />
      </label>
    </section>

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">显示</h2>
        <p class="block-desc">字体大小与列表密度</p>
      </header>

      <div class="seg-group">
        <button
          v-for="opt in [{ id: 'small', label: '小' }, { id: 'standard', label: '标准' }, { id: 'large', label: '大' }]"
          :key="opt.id"
          class="seg-btn"
          :class="{ active: display.fontSize === opt.id }"
          @click="display.setFontSize(opt.id as FontSize)"
        >
          {{ opt.label }}
        </button>
      </div>

      <label class="row" style="margin-top: 10px;">
        <div class="row-text">
          <div class="row-label">紧凑模式</div>
          <div class="row-desc">缩小列表行高与间距，单屏显示更多条目</div>
        </div>
        <input
          :checked="display.compact"
          type="checkbox"
          class="toggle"
          @change="display.setCompact(($event.target as HTMLInputElement).checked)"
        />
      </label>
    </section>

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">自动清理</h2>
        <p class="block-desc">超过阈值的非收藏非置顶条目会被自动删除（留空表示不限制）</p>
      </header>

      <div class="grid-2">
        <label class="field">
          <span class="field-label">保留天数</span>
          <input
            v-model.number="maxAgeDays"
            type="number"
            min="0"
            placeholder="如 30"
            class="field-input"
          />
          <span class="field-hint">天</span>
        </label>
        <label class="field">
          <span class="field-label">最大条数</span>
          <input
            v-model.number="maxCount"
            type="number"
            min="0"
            placeholder="如 1000"
            class="field-input"
          />
          <span class="field-hint">条</span>
        </label>
        <label class="field">
          <span class="field-label">最大占用</span>
          <input
            v-model.number="maxBytesMB"
            type="number"
            min="0"
            placeholder="如 500"
            class="field-input"
          />
          <span class="field-hint">MB</span>
        </label>
      </div>

      <div class="row-actions">
        <button class="action-btn" @click="cleanupNow">立即清理</button>
        <button class="action-btn primary" @click="saveAll">保存设置</button>
      </div>
    </section>

    <section class="block">
      <header class="block-header">
        <h2 class="block-title">忽略名单</h2>
        <p class="block-desc">来自这些应用的复制内容不会被记录（按应用名匹配，区分大小写）</p>
      </header>

      <div class="ignore-input">
        <input
          v-model="newIgnoreApp"
          type="text"
          placeholder="如 1Password"
          class="field-input flex"
          @keydown.enter.prevent="addIgnoreApp"
        />
        <button class="action-btn" @click="addIgnoreApp">添加</button>
      </div>

      <div v-if="ignoreApps.length === 0" class="empty-list">暂无忽略应用</div>
      <div v-else class="tag-list">
        <span v-for="name in ignoreApps" :key="name" class="tag">
          {{ name }}
          <button class="tag-x" type="button" @click="removeIgnoreApp(name)">×</button>
        </span>
      </div>

      <div class="row-actions">
        <button class="action-btn primary" @click="saveAll">保存设置</button>
      </div>
    </section>

    <section class="block ai-block">
      <header class="block-header">
        <h2 class="block-title">AI 助手</h2>
        <p class="block-desc">配置大语言模型，启用右键 AI 操作（翻译 / 总结 / 润色等）</p>
      </header>

      <div class="ai-form">
        <div class="ai-row">
          <label class="ai-label">Provider</label>
          <select v-model="llmProvider" class="field-input ai-select">
            <option v-for="opt in providerOptions" :key="opt.id" :value="opt.id">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <div class="ai-row">
          <label class="ai-label">API Key</label>
          <input
            v-model="llmApiKey"
            type="password"
            class="field-input"
            :placeholder="llmProvider === 'ollama' ? '本地模型无需 Key' : 'sk-...'"
            :disabled="llmProvider === 'ollama'"
          />
        </div>

        <div class="ai-row">
          <label class="ai-label">Base URL</label>
          <input
            v-model="llmBaseUrl"
            type="text"
            class="field-input"
            :placeholder="currentProvider.default_base"
          />
        </div>

        <div class="ai-row">
          <label class="ai-label">Model</label>
          <input
            v-model="llmModel"
            type="text"
            class="field-input"
            :placeholder="currentProvider.default_model"
          />
        </div>

        <div v-if="testResult" class="ai-test" :class="{ ok: testResult.ok, fail: !testResult.ok }">
          {{ testResult.message }}
        </div>

        <div class="ai-actions">
          <button class="action-btn" :disabled="testing" @click="testConnection">
            {{ testing ? "测试中..." : "测试连接" }}
          </button>
          <button class="action-btn primary" @click="saveAll">保存设置</button>
        </div>
      </div>
    </section>

    <footer class="settings-footer">
      <span>Sparkbox v0.1.0 · 配置存储于本地浏览器</span>
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
  gap: 10px;
  padding: 12px 34px 12px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: all var(--transition);
  min-height: 56px;
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
  width: 32px;
  height: 32px;
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
  width: 16px;
  height: 16px;
}

.mode-text {
  flex: 1;
  min-width: 0;
}

.mode-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-bright);
  line-height: 1.3;
  white-space: nowrap;
}

.mode-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  margin-top: 2px;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  cursor: pointer;
  transition: all var(--transition);
}

.row:hover {
  border-color: var(--border-strong);
}

.row-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.row-label {
  font-size: 13px;
  color: var(--text-bright);
  font-weight: 500;
}

.row-desc {
  font-size: 11.5px;
  color: var(--text-muted);
}

.toggle {
  appearance: none;
  width: 36px;
  height: 20px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  position: relative;
  cursor: pointer;
  transition: background var(--transition);
  flex-shrink: 0;
}

.toggle::after {
  content: "";
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--text-bright);
  transition: transform var(--transition);
}

.toggle:checked {
  background: var(--accent);
}

.toggle:checked::after {
  transform: translateX(16px);
}

.ai-block {
  border: 1px dashed var(--border-strong);
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.04), transparent 60%);
}

.ai-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ai-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ai-label {
  font-size: 11.5px;
  color: var(--text-muted);
  font-weight: 500;
}

.ai-select {
  appearance: none;
  background-image: linear-gradient(45deg, transparent 50%, var(--text-muted) 50%),
    linear-gradient(135deg, var(--text-muted) 50%, transparent 50%);
  background-position: calc(100% - 16px) 50%, calc(100% - 11px) 50%;
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
  padding-right: 32px;
}

.ai-test {
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  line-height: 1.5;
  word-break: break-word;
}

.ai-test.ok {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.ai-test.fail {
  background: rgba(239, 68, 68, 0.1);
  color: var(--danger);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.ai-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;
}

.ai-placeholder {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
}

.ai-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.12);
  color: var(--accent);
  flex-shrink: 0;
}

.ai-icon svg {
  width: 22px;
  height: 22px;
}

.ai-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.ai-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-bright);
}

.ai-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  letter-spacing: 0.02em;
}

.grid-2 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

@media (max-width: 720px) {
  .grid-2 {
    grid-template-columns: 1fr;
  }
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  position: relative;
}

.field-label {
  font-size: 11.5px;
  color: var(--text-muted);
}

.field-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elev-1);
  color: var(--text-bright);
  font-size: 13px;
  outline: none;
  transition: all var(--transition);
}

.field-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.field-hint {
  position: absolute;
  right: 12px;
  bottom: 16px;
  font-size: 11px;
  color: var(--text-muted);
  pointer-events: none;
}

.row-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  justify-content: flex-end;
}

.action-btn {
  padding: 7px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elev-2);
  color: var(--text-dim);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}

.action-btn:hover {
  border-color: var(--border-strong);
  color: var(--text);
}

.action-btn.primary {
  border-color: var(--accent);
  background: var(--accent);
  color: white;
}

.action-btn.primary:hover {
  filter: brightness(1.08);
}

.ignore-input {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}

.ignore-input .field-input.flex {
  flex: 1;
}

.empty-list {
  padding: 16px;
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
  border: 1px dashed var(--border);
  border-radius: var(--radius);
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev-2);
  min-height: 40px;
}

.tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-bright);
  font-size: 12px;
  border: 1px solid var(--border);
}

.tag-x {
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  padding: 0;
  display: inline-flex;
  align-items: center;
}

.tag-x:hover {
  color: var(--danger);
}

.seg-group {
  display: inline-flex;
  padding: 3px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  gap: 2px;
}

.seg-btn {
  padding: 6px 18px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--text-dim);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}

.seg-btn:hover {
  color: var(--text);
}

.seg-btn.active {
  background: var(--accent);
  color: white;
  box-shadow: 0 2px 6px var(--accent-glow);
}
</style>
