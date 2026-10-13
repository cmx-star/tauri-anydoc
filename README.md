# AnyDoc Markdown Converter (桌面端应用)

基于 [Firecrawl anydoc](https://github.com/firecrawl/anydoc) 核心与 Tauri v2 构建的跨平台高性能桌面应用，支持将各类文档与文件极速转换为标准的 GitHub-Flavored Markdown (GFM)。

![AnyDoc Converter](src-tauri/icons/icon.png)

---

## 🌟 核心特性

- **多格式毫秒级转换**：
  - **Office 办公套件**：Word (`.docx`, `.doc`)、PowerPoint (`.pptx`, `.ppt`)、Excel (`.xlsx`, `.xls`)。
  - **电子书与通用版式**：PDF (`.pdf`)、EPUB (`.epub`)、RTF (`.rtf`)。
  - **表格与开放格式**：CSV (`.csv`)、OpenDocument (`.odt`, `.ods`, `.odp`)。
- **智能兜底转换**：
  - 自动识别编程代码文件（`.rs`, `.py`, `.js`, `.ts`, `.json`, `.yaml`, `.toml`, `.go`, `.c`, `.cpp` 等）并包装为带语法高亮的 Markdown 代码块。
  - 普通纯文本文件自动保留原格式内容（不插入额外文件名标题）。
  - 非文本二进制文件自动提取元信息表格（文件名、大小、扩展名提示）。
- **桌面端原生体验**：
  - **拖拽即转换**：支持将单文件、多个文件或整个文件夹拖入窗口自动展开。
  - **双栏分屏预览**：实时切换“富文本渲染预览”（Atom One Dark 代码高亮、表格渲染）与“原始 Markdown 源码”。
  - **批量与导出能力**：
    - 一键复制 Markdown 到剪贴板。
    - 单文件另存为 `.md`。
    - 批量导出所有结果至目标目录。
    - 合并所有文件导出为一个总 Markdown 文档。

---

## 🛠️ 技术架构

- **后端引擎**：Rust 2021 + `anydoc` 0.2.4 + `tauri` 2.x
- **前端界面**：Vue 3 + TypeScript + Vite 6 + `marked` 15 + `highlight.js` + `lucide-vue-next`

---

## 🚀 启动与使用指南

### 1. 开发模式启动应用

```bash
# 启动桌面端调试窗口
pnpm tauri dev
```

### 2. 本地构建安装包

```bash
# 本地打包构建 macOS .dmg / .app 安装包
pnpm tauri build
```

---

## 📦 自动化发布流程 (Tag Release)

项目配置了完整的 **GitHub Actions 自动化发版流水线**（`.github/workflows/release.yml`），只需推送以 `v` 开头的 Tag 即可触发跨平台编译并发布 GitHub Release。

### 1. 使用内置脚本自动化发版

项目内置了 `scripts/release.js` 工具，会自动完成以下操作：
1. 同步更新 `package.json`、`src-tauri/Cargo.toml` 与 `src-tauri/tauri.conf.json` 三处版本号；
2. 自动运行 Rust 单元测试与前端生产构建检查；
3. 自动提交并创建对应的 Git Tag（如 `v0.1.1`）。

```bash
# 升级补丁版本 (例如 0.1.0 -> 0.1.1)
pnpm run release patch

# 升级次版本号 (例如 0.1.0 -> 0.2.0)
pnpm run release minor

# 指定版本号 (例如 1.0.0)
pnpm run release 1.0.0
```

### 2. 推送 Tag 触发 GitHub 跨平台发版

打上 Tag 后，只需将代码与标签推送到 GitHub：

```bash
git push origin HEAD
git push origin v0.1.1
```

GitHub Actions 将自动并行执行以下矩阵构建：
- **macOS (Apple Silicon & Intel)**：生成 `.dmg` 安装包
- **Windows**：生成 `.msi` 与 `.exe` 安装包
- **Linux (Ubuntu)**：生成 `.deb` 与 `.AppImage` 安装包

构建完成后将自动在 GitHub Releases 页面创建新发布版本并挂载所有下载安装包。
