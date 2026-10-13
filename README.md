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
  - 普通纯文本文件自动保留格式。
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

### 2. 构建独立桌面安装包

```bash
# 打包构建 macOS .dmg / .app 安装包
pnpm tauri build
```

打包完成后，二进制或安装包将输出在 `src-tauri/target/release/bundle/` 目录下。

### 3. 运行后端单元测试

```bash
cd src-tauri && cargo test
```
