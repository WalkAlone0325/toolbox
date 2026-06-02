# Toolbox - 跨平台桌面剪贴板管理工具

基于 Tauri 2 的跨平台桌面剪贴板管理工具，支持文本、图片等多种剪贴板内容的存储、检索和管理。

## 功能特性

- ✅ 自动监听剪贴板变化
- ✅ 文本内容管理（搜索、删除、收藏、置顶）
- ✅ 图片内容管理（自动生成缩略图）
- ✅ 全文搜索支持
- ✅ 跨平台支持（Windows、macOS、Linux）

## 技术栈

- **前端**: Vue 3.5 + TypeScript 5.7 + Vite 6 + Pinia 2
- **后端**: Rust (edition 2021) + Tauri 2
- **数据库**: SQLite (Rusqlite) + FTS5 全文搜索
- **剪贴板**: Arboard
- **图片处理**: Image crate (PNG 格式)

## 快速开始

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri:dev    # 启动 Tauri 开发模式
```

### 构建

```bash
pnpm tauri:build  # 构建 Tauri 应用
```

## 开发命令

### 前端命令
```bash
pnpm dev               # 启动 Vite 开发服务器
pnpm build             # 构建 Vite 前端
pnpm type-check        # TypeScript 类型检查
```

### 后端命令
```bash
cargo build            # 编译 Rust 代码
cargo check            # 代码检查
cargo run              # 运行 Rust 应用
```

## 项目结构

```
toolbox/
├── src/                      # 前端代码
│   ├── App.vue              # 主应用组件
│   ├── layout/              # 布局组件
│   ├── tools/               # 工具模块
│   ├── stores/              # Pinia 状态管理
│   ├── composables/         # Vue 组合式函数
│   └── types/               # TypeScript 类型定义
├── src-tauri/               # 后端代码
│   ├── src/
│   │   ├── core/            # 核心模块
│   │   │   ├── db.rs        # 数据库操作
│   │   │   └── clipboard.rs # 剪贴板监控
│   │   └── tools/           # 工具命令
│   └── tauri.conf.json      # Tauri 配置
├── package.json             # 前端依赖
└── Cargo.toml               # 后端依赖
```

## 数据存储

- 图片存储位置: `app_data_dir/images/`
- 缩略图存储位置: `app_data_dir/thumbs/`
- 数据库文件: `app_data_dir/toolbox.db`

## 功能说明

### 剪贴板历史管理
- 自动记录剪贴板历史
- 支持全文搜索
- 支持按类型过滤（文本/图片）
- 支持收藏和置顶功能
- 支持粘贴任意历史条目

### 剪贴板监控
- 后台线程持续监听剪贴板变化
- 使用 SHA256 哈希值去重
- 自动处理文本和图片内容
- 支持多种来源应用的剪贴板内容

## 开发指南

详见 [CLAUDE.md](./CLAUDE.md)

## License

MIT
