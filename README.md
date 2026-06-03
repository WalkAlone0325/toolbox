<div align="center">

# Sparkbox

**一款跨平台桌面剪贴板管理工具**

文本 · 图片 · 文件 · 全文搜索 · 主题自定义 · AI 助手

[![Release](https://img.shields.io/github/v/release/WalkAlone0325/toolbox?style=flat-square&logo=github&color=blue)](https://github.com/WalkAlone0325/toolbox/releases)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey?style=flat-square)]()
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange?style=flat-square&logo=tauri)](https://tauri.app)

</div>

---

## 📖 简介

**Sparkbox** 是一款基于 [Tauri 2](https://tauri.app) 构建的跨平台桌面剪贴板管理工具。它持续监听系统剪贴板，将每次复制的文本、图片、文件自动归档为可检索的历史记录，并提供全文搜索、收藏置顶、自动清理、AI 智能处理等高级能力。

> 🎉 **v0.1.0 已发布** → [立即下载](https://github.com/WalkAlone0325/toolbox/releases/tag/v0.1.0)

## ✨ 功能特性

### 📋 剪贴板历史
- **多类型支持**：文本、图片、文件列表自动分类存储
- **全文搜索**：基于 SQLite FTS5 的中文/英文全文检索
- **快速过滤**：按类型（文本/图片）一键筛选
- **收藏置顶**：常用内容固定在顶部，不被清理
- **来源追踪**：记录每条历史的来源应用
- **去重**：基于 SHA256 哈希自动去重

### 🎨 主题与外观
- **三种模式**：跟随系统 / 浅色 / 深色
- **多套预设**：内置 6+ 套配色风格
- **自定义主色**：8 色预设 + 自由取色器
- **显示密度**：紧凑 / 标准 / 舒展三档

### 🧹 智能清理
- **按时间**：超过 N 天自动清理
- **按数量**：保留最近 N 条
- **按体积**：超过总大小自动清理
- **忽略应用**：避免密码管理器等敏感工具污染历史

### 🤖 AI 助手（实验性）
- **多供应商**：支持 OpenAI / Anthropic Claude / Ollama 本地
- **连通性测试**：内置一键测试，配置错误即时反馈
- **本地优先**：Ollama 模式下数据不出本机

### ⚡ 体验优化
- **全局快捷键**：唤起窗口，无需鼠标
- **窗口失焦自动隐藏**：类 PasteBot 风格，按 Esc 或点其他应用自动收起
- **右键菜单**：复制、粘贴、收藏、置顶、删除一键操作
- **缩略图预览**：图片自动生成 64×64 缩略图

## 📦 安装

### 方式一：下载预编译包（推荐）

前往 [Releases](https://github.com/WalkAlone0325/toolbox/releases) 下载对应平台的安装包：

| 平台 | 文件 | 说明 |
|---|---|---|
| macOS (Apple Silicon) | `Sparkbox_*_aarch64.dmg` | M1/M2/M3/M4 系列 |
| macOS (Intel) | 暂未提供，请源码构建 | - |
| Windows / Linux | 暂未提供，请源码构建 | - |

**macOS 安装提示**：首次打开如提示"无法验证开发者"，请到 **系统设置 → 隐私与安全性** 点击"仍要打开"。

### 方式二：源码构建

```bash
git clone https://github.com/WalkAlone0325/toolbox.git
cd toolbox
pnpm install
pnpm tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 🛠️ 开发

### 环境要求

- [Node.js](https://nodejs.org/) ≥ 18
- [pnpm](https://pnpm.io/) ≥ 8
- [Rust](https://www.rust-lang.org/) ≥ 1.75（推荐 rustup 稳定版）
- 平台依赖：
  - **macOS**：Xcode Command Line Tools
  - **Windows**：Microsoft Visual C++ Build Tools / WebView2
  - **Linux**：`webkit2gtk-4.1`、`libssl-dev` 等（详见 [Tauri 文档](https://tauri.app/start/prerequisites/)）

### 常用命令

```bash
pnpm install           # 安装前端依赖
pnpm tauri:dev         # 启动 Tauri 开发模式（前后端联调）
pnpm tauri:build       # 构建生产版本（输出安装包）
pnpm type-check        # TypeScript 类型检查

# 仅前端
pnpm dev               # 启动 Vite 开发服务器
pnpm build             # 构建前端产物

# 仅后端（在 src-tauri/ 下执行）
cargo check            # 代码检查
cargo clippy           # 代码规范检查
cargo test             # 运行测试
```

### 项目结构

```
toolbox/
├── src/                          # 前端代码（Vue 3 + TS）
│   ├── App.vue                   # 主应用组件
│   ├── main.ts                   # 应用入口
│   ├── layout/                   # 布局组件（Sidebar、ToolContainer）
│   ├── components/               # 通用组件（ContextMenu、Toast）
│   ├── tools/                    # 工具模块（ClipboardManager、Settings）
│   ├── stores/                   # Pinia 状态管理
│   │   ├── clipboard.ts          # 剪贴板状态
│   │   ├── theme.ts              # 主题/外观
│   │   ├── display.ts            # 显示密度/字号
│   │   ├── sparkbox.ts           # 全局工具状态
│   │   └── toast.ts              # 消息提示
│   ├── composables/              # Vue 组合式函数
│   └── types/                    # TypeScript 类型定义
│
├── src-tauri/                    # 后端代码（Rust）
│   ├── src/
│   │   ├── lib.rs                # Tauri 应用入口
│   │   ├── core/                 # 核心模块
│   │   │   ├── db.rs             # SQLite + FTS5
│   │   │   ├── clipboard.rs      # 剪贴板监控
│   │   │   ├── settings.rs       # 应用配置持久化
│   │   │   └── llm/              # AI 供应商封装
│   │   │       ├── openai.rs
│   │   │       ├── anthropic.rs
│   │   │       └── ollama.rs
│   │   └── tools/                # Tauri 命令
│   │       ├── clipboard.rs      # 剪贴板管理命令
│   │       └── ai.rs             # AI 相关命令
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
│
├── package.json                  # 前端依赖
└── README.md
```

## 🧱 技术栈

| 层级 | 技术 |
|---|---|
| 前端 | Vue 3.5 · TypeScript 5.7 · Vite 6 · Pinia 2 |
| 后端 | Rust (edition 2021) · Tauri 2 |
| 数据库 | SQLite (Rusqlite) + FTS5 全文搜索 |
| 剪贴板 | Arboard · macOS 原生 NSPasteboard |
| 图片处理 | Image crate (PNG) |
| AI 集成 | reqwest + tokio（异步 HTTP，支持 OpenAI / Anthropic / Ollama） |

## 📐 数据存储

| 类型 | 路径 |
|---|---|
| 数据库 | `<app_data_dir>/sparkbox.db` |
| 原图 | `<app_data_dir>/images/` |
| 缩略图 | `<app_data_dir>/thumbs/` |
| 配置 | `<app_data_dir>/settings.json` |

`<app_data_dir>` 在各平台默认为：
- **macOS**：`~/Library/Application Support/Sparkbox/`
- **Windows**：`%APPDATA%\Sparkbox\`
- **Linux**：`~/.local/share/Sparkbox/`

## 🗺️ 路线图

- [x] 剪贴板历史 + 全文搜索
- [x] 主题与外观自定义
- [x] 自动清理 + 忽略应用
- [x] AI 供应商配置（OpenAI / Anthropic / Ollama）
- [x] 全局快捷键
- [ ] AI 智能打标 / 翻译 / 总结
- [ ] 跨条目 AI 问答
- [ ] 多设备同步（iCloud / WebDAV）
- [ ] Windows / Linux 官方包
- [ ] 国际化（i18n）

详见 [Issues](https://github.com/WalkAlone0325/toolbox/issues)。

## 🤝 贡献

欢迎提 Issue 或 Pull Request。

提 PR 前请确保：
1. `pnpm type-check` 通过
2. `cargo clippy` 无警告
3. 新功能有合理的设计说明

## 📄 License

[MIT](./LICENSE)
