# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Tauri 2 的跨平台桌面剪贴板管理工具，支持文本、图片等多种剪贴板内容的存储、检索和管理。

## 开发命令

### 一键命令（推荐）

```bash
pnpm tauri:dev          # 启动 Tauri 开发模式（前后端联调）
pnpm tauri:build        # 构建生产版本（输出安装包）
pnpm tauri:run          # 运行已构建的应用
pnpm type-check         # TypeScript 类型检查
```

### 前端命令
```bash
pnpm dev                # 仅启动 Vite 前端开发服务器
pnpm build              # 仅构建 Vite 前端产物
```

### 后端命令（在 src-tauri/ 目录下执行）
```bash
cargo build             # 编译 Rust 代码
cargo check             # 代码检查（不生成产物）
cargo run               # 直接运行 Rust 二进制
cargo test              # 运行测试
cargo clippy            # 代码规范检查
cargo fmt               # 格式化代码
```

### Tauri CLI 原生命令
```bash
pnpm tauri dev          # 开发模式
pnpm tauri build        # 构建生产版本
pnpm tauri build --debug   # 构建调试版本（更快编译）
pnpm tauri info         # 查看环境与配置信息
```

### 清理命令
```bash
pnpm store prune        # 清理 pnpm 存储
cargo clean             # 在 src-tauri/ 下清理构建缓存
rm -rf node_modules dist   # 清理前端产物
```


## 技术栈

- **前端**: Vue 3.5 + TypeScript 5.7 + Vite 6 + Pinia 2
- **后端**: Rust (edition 2021) + Tauri 2
- **状态管理**: Pinia
- **数据库**: SQLite (Rusqlite) + FTS5 全文搜索
- **剪贴板**: Arboard
- **图片处理**: Image crate (PNG 格式)

## 架构说明

### 前端架构 (src/)

- `App.vue`: 主应用组件，包含侧边栏和工具容器
- `layout/`: 布局组件
  - `Sidebar.vue`: 工具选择侧边栏
  - `ToolContainer.vue`: 工具显示容器
- `tools/`: 工具模块
  - `ClipboardManager.vue`: 剪贴板管理组件
  - `index.ts`: 工具注册表，定义可用工具
- `stores/`: Pinia 状态管理
  - `clipboard.ts`: 剪贴板相关状态
  - `toolbox.ts`: 工具箱状态
- `composables/`: Vue 组合式函数
  - `useClipboard.ts`: 剪贴板操作钩子
- `types/`: TypeScript 类型定义

### 后端架构 (src-tauri/src/)

- `lib.rs`: Tauri 应用入口，注册命令和生命周期钩子
- `core/`: 核心模块
  - `db.rs`: SQLite 数据库操作，包含全文搜索 FTS5
  - `clipboard.rs`: 系统剪贴板监控和操作
- `tools/`: 工具命令模块
  - `clipboard.rs`: 剪贴板管理命令

## 数据模型

剪贴板条目包含以下字段：
- `id`: 主键
- `hash`: 唯一哈希值
- `type`: 条目类型（text/image）
- `text_val`: 文本内容
- `image_path`: 图片存储路径
- `thumb_path`: 缩略图路径
- `file_list`: 文件列表
- `source_app`: 来源应用
- `byte_size`: 字节大小
- `fav`: 是否收藏
- `pinned`: 是否置顶
- `created_at`: 创建时间戳
- `updated_at`: 更新时间戳

## 工作流程

1. 应用启动时初始化 SQLite 数据库，创建索引和 FTS5 虚拟表
2. 后台线程启动剪贴板监控（每 500ms 检查一次）
3. 检测到剪贴板变化时，计算内容的 SHA256 哈希值
4. 前端通过 Tauri 命令与后端通信
5. 数据查询支持全文搜索和类型过滤
6. 图片存储在 `app_data_dir/images/` 目录
7. 缩略图存储在 `app_data_dir/thumbs/` 目录

## 核心模块说明

### 剪贴板监控 (core/clipboard.rs)

- `start_monitoring()`: 后台线程启动剪贴板监控
- `read_clipboard()`: 读取剪贴板内容（支持文本和图片）
- `compute_hash()`: 计算 SHA256 哈希值用于去重
- Mac OS 平台使用 NSPasteboard changeCount 检测变化

### 数据库 (core/db.rs)

- `Database`: 数据库封装结构
- `has_entry()`: 检查条目是否已存在
- `insert_entry()`: 插入剪贴板条目
- `get_history()`: 查询历史记录（支持全文搜索和类型过滤）
- `toggle_favorite()` / `toggle_pin()`: 切换收藏和置顶状态
- 使用 FTS5 全文搜索虚拟表实现搜索功能

### Tauri 命令 (tools/clipboard.rs)

- `get_history()`: 获取剪贴板历史记录
- `delete_entry()`: 删除指定条目
- `toggle_favorite()`: 切换收藏状态
- `toggle_pin()`: 切换置顶状态
- `paste_entry()`: 粘贴指定条目（支持文本和图片）

## 新增工具流程

1. 在 `src/tools/` 创建新组件 `YourToolName.vue`
2. 在 `src/tools/index.ts` 注册工具
3. 在 `src/types/` 添加工具类型定义
4. 在 `src/stores/toolbox.ts` 添加工具状态
5. 组件需遵循工具容器的设计模式

## 关键依赖

- `tauri`: Tauri 核心库
- `tauri-plugin-shell`: Shell 集成
- `rusqlite`: SQLite 数据库
- `arboard`: 跨平台剪贴板
- `chrono`: 时间处理
- `serde`: 序列化/反序列化
