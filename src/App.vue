<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { marked } from "marked";
import type { TaskItem, ConvertResult, BatchConvertSummary } from "./types";
import {
  FileText,
  FolderOpen,
  Play,
  Trash2,
  Copy,
  Download,
  FolderDown,
  Layers,
  CheckCircle2,
  AlertCircle,
  Clock,
  Sparkles,
  FileCode,
  FileSpreadsheet,
  FileBox,
  Eye,
  Code
} from "lucide-vue-next";

const tasks = ref<TaskItem[]>([]);
const selectedId = ref<string | null>(null);
const isConvertingAll = ref(false);
const viewMode = ref<"rendered" | "raw">("rendered");
const isDraggingOver = ref(false);
const toastMessage = ref<string | null>(null);

const selectedTask = computed(() => {
  return tasks.value.find((t) => t.id === selectedId.value) || null;
});

const stats = computed(() => {
  const total = tasks.value.length;
  const completed = tasks.value.filter((t) => t.status === "success").length;
  const failed = tasks.value.filter((t) => t.status === "error").length;
  const totalDuration = tasks.value.reduce((acc, cur) => acc + (cur.duration_ms || 0), 0);
  return { total, completed, failed, totalDuration };
});

function showToast(msg: string) {
  toastMessage.value = msg;
  setTimeout(() => {
    if (toastMessage.value === msg) {
      toastMessage.value = null;
    }
  }, 2500);
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function getFormatCategory(ext: string): { label: string; color: string; icon: any } {
  const e = ext.toLowerCase();
  if (["doc", "docx", "odt", "rtf"].includes(e)) {
    return { label: "WORD", color: "#38bdf8", icon: FileText };
  }
  if (["xls", "xlsx", "csv", "ods"].includes(e)) {
    return { label: "SHEET", color: "#34d399", icon: FileSpreadsheet };
  }
  if (["ppt", "pptx", "odp"].includes(e)) {
    return { label: "SLIDE", color: "#fb923c", icon: FileBox };
  }
  if (["pdf"].includes(e)) {
    return { label: "PDF", color: "#f87171", icon: FileText };
  }
  if (["epub"].includes(e)) {
    return { label: "EPUB", color: "#c084fc", icon: FileText };
  }
  if (["rs", "py", "js", "ts", "json", "html", "css", "yaml", "toml", "go", "c", "cpp", "java"].includes(e)) {
    return { label: "CODE", color: "#a78bfa", icon: FileCode };
  }
  return { label: ext ? ext.toUpperCase() : "FILE", color: "#94a3b8", icon: FileText };
}

// 添加文件到列表
function addFiles(filePaths: string[]) {
  for (const path of filePaths) {
    if (tasks.value.some((t) => t.file_path === path)) {
      continue;
    }
    const name = path.split("/").pop() || "unknown";
    const ext = name.includes(".") ? name.split(".").pop()?.toLowerCase() || "" : "";
    const id = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    tasks.value.push({
      id,
      file_path: path,
      file_name: name,
      extension: ext,
      file_size: 0,
      success: false,
      markdown: "",
      error_message: null,
      engine: "pending",
      duration_ms: 0,
      status: "idle",
    });

    if (!selectedId.value) {
      selectedId.value = id;
    }
  }
}

// 选择文件对话框
async function handleSelectFiles() {
  try {
    const selected = await open({
      multiple: true,
      title: "选择待转换文件",
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      addFiles(paths);
    }
  } catch (err) {
    console.error("选择文件失败:", err);
  }
}

// 选择文件夹对话框
async function handleSelectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择待扫描目录",
    });
    if (selected && typeof selected === "string") {
      // 通过后端批量转换命令递归展开目录并转换
      const summary = await invoke<BatchConvertSummary>("convert_files_batch", {
        paths: [selected],
      });
      for (const res of summary.results) {
        if (!tasks.value.some((t) => t.file_path === res.file_path)) {
          tasks.value.push({
            id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
            ...res,
            status: res.success ? "success" : "error",
          });
        }
      }
      if (!selectedId.value && tasks.value.length > 0) {
        selectedId.value = tasks.value[0].id;
      }
      showToast(`扫描目录完成，发现并转换 ${summary.total} 个文件`);
    }
  } catch (err) {
    console.error("选择目录失败:", err);
  }
}

// 转换单项
async function convertSingleTask(task: TaskItem) {
  task.status = "converting";
  try {
    const res = await invoke<ConvertResult>("convert_file", {
      path: task.file_path,
    });
    task.file_size = res.file_size;
    task.success = res.success;
    task.markdown = res.markdown;
    task.error_message = res.error_message;
    task.engine = res.engine;
    task.duration_ms = res.duration_ms;
    task.status = res.success ? "success" : "error";
  } catch (e: any) {
    task.status = "error";
    task.error_message = String(e);
  }
}

// 转换全部
async function handleConvertAll() {
  if (isConvertingAll.value || tasks.value.length === 0) return;
  isConvertingAll.value = true;

  for (const task of tasks.value) {
    if (task.status !== "success") {
      await convertSingleTask(task);
    }
  }

  isConvertingAll.value = false;
  showToast("批量转换已完成！");
}

// 复制 Markdown 到剪贴板
async function copyMarkdown(text: string) {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    showToast("已成功复制 Markdown 到剪贴板！");
  } catch (e) {
    showToast("复制失败，请手动选择复制");
  }
}

// 单文件另存为
async function handleSaveSingle(task: TaskItem) {
  if (!task.markdown) return;
  try {
    const defaultName = task.file_name.replace(/\.[^/.]+$/, "") + ".md";
    const targetPath = await save({
      filters: [{ name: "Markdown 文档", extensions: ["md"] }],
      defaultPath: defaultName,
    });
    if (targetPath) {
      await invoke("save_markdown", {
        targetPath,
        content: task.markdown,
      });
      showToast(`已成功保存至: ${targetPath}`);
    }
  } catch (e) {
    showToast(`保存失败: ${e}`);
  }
}

// 批量保存所有文件到目录
async function handleSaveBatch() {
  const completed = tasks.value.filter((t) => t.status === "success" && t.markdown);
  if (completed.length === 0) {
    showToast("当前没有可导出的已转换文件");
    return;
  }

  try {
    const targetDir = await open({
      directory: true,
      multiple: false,
      title: "选择 Markdown 输出目录",
    });
    if (targetDir && typeof targetDir === "string") {
      const savedCount = await invoke<number>("save_batch_markdown", {
        targetDir,
        results: completed,
      });
      showToast(`已成功导出 ${savedCount} 个 Markdown 文件！`);
    }
  } catch (e) {
    showToast(`导出失败: ${e}`);
  }
}

// 合并导出为单文件
async function handleExportCombined() {
  const completed = tasks.value.filter((t) => t.status === "success" && t.markdown);
  if (completed.length === 0) {
    showToast("当前没有可合并的已转换文件");
    return;
  }

  try {
    const targetPath = await save({
      filters: [{ name: "Markdown 文档", extensions: ["md"] }],
      defaultPath: "combined_documents.md",
    });
    if (targetPath) {
      await invoke("export_combined_markdown", {
        targetPath,
        results: completed,
      });
      showToast(`已合并导出至: ${targetPath}`);
    }
  } catch (e) {
    showToast(`合并导出失败: ${e}`);
  }
}

// 清空列表
function clearAll() {
  tasks.value = [];
  selectedId.value = null;
  showToast("任务列表已清空");
}

// 移除单个任务
function removeTask(id: string, event: Event) {
  event.stopPropagation();
  tasks.value = tasks.value.filter((t) => t.id !== id);
  if (selectedId.value === id) {
    selectedId.value = tasks.value[0]?.id || null;
  }
}

// 计算渲染后的 HTML
const renderedHtml = computed(() => {
  if (!selectedTask.value || !selectedTask.value.markdown) return "";
  try {
    return marked.parse(selectedTask.value.markdown) as string;
  } catch (e) {
    return `<p style="color: #f87171">渲染失败: ${e}</p>`;
  }
});

// HTML 拖拽与 Tauri Webview 拖拽监听
onMounted(async () => {
  try {
    const webview = getCurrentWebviewWindow();
    await webview.onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        isDraggingOver.value = true;
      } else if (event.payload.type === "drop") {
        isDraggingOver.value = false;
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          addFiles(paths);
          // 自动触发新加入文件的转换
          for (const p of paths) {
            const task = tasks.value.find((t) => t.file_path === p);
            if (task && task.status === "idle") {
              convertSingleTask(task);
            }
          }
        }
      } else {
        isDraggingOver.value = false;
      }
    });
  } catch (err) {
    console.warn("Tauri drag-drop 注册提示 (若在浏览器中运行可忽略):", err);
  }
});
</script>

<template>
  <div class="app-container">
    <!-- 顶部导航栏 -->
    <header class="app-header">
      <div class="header-left">
        <div class="logo-box">
          <Sparkles class="logo-icon" />
        </div>
        <div>
          <h1 class="app-title">AnyDoc <span class="badge">PRO</span></h1>
          <p class="app-subtitle">基于 anydoc 核心 · 毫秒级多格式万能文档转 Markdown</p>
        </div>
      </div>

      <div class="header-actions">
        <!-- 统计面板 -->
        <div class="stats-capsule">
          <span class="stat-item">全部 <b>{{ stats.total }}</b></span>
          <span class="stat-divider">/</span>
          <span class="stat-item text-green">完成 <b>{{ stats.completed }}</b></span>
          <span class="stat-divider">/</span>
          <span class="stat-item text-red">失败 <b>{{ stats.failed }}</b></span>
          <span class="stat-divider">|</span>
          <span class="stat-item text-muted"><Clock :size="12" /> {{ stats.totalDuration }}ms</span>
        </div>

        <button class="btn btn-secondary" @click="handleSelectFiles">
          <FileText :size="15" /> 添加文件
        </button>
        <button class="btn btn-secondary" @click="handleSelectDirectory">
          <FolderOpen :size="15" /> 扫描目录
        </button>
        <button
          class="btn btn-primary"
          :disabled="isConvertingAll || tasks.length === 0"
          @click="handleConvertAll"
        >
          <Play :size="15" /> {{ isConvertingAll ? "正在转换..." : "一键转换全部" }}
        </button>
        <button
          v-if="tasks.length > 0"
          class="btn btn-danger-icon"
          title="清空全部任务"
          @click="clearAll"
        >
          <Trash2 :size="15" />
        </button>
      </div>
    </header>

    <!-- 主工作区：左侧列表 + 右侧预览 -->
    <main class="app-main">
      <!-- 左侧：文件管理面板 -->
      <section class="sidebar-panel">
        <!-- 拖放引导区域 -->
        <div
          class="drop-zone"
          :class="{ 'is-dragging': isDraggingOver }"
          @click="handleSelectFiles"
        >
          <Layers class="drop-icon" :size="28" />
          <div class="drop-text">
            <strong>拖拽文件或文件夹至此</strong>
            <span>支持 Word, PPT, Excel, PDF, EPUB, RTF, CSV, 源码</span>
          </div>
        </div>

        <!-- 任务列表 -->
        <div class="tasks-scroll">
          <div v-if="tasks.length === 0" class="empty-list">
            <p>暂无待处理文件</p>
            <span>点击上方按钮或直接拖入文件开始转换</span>
          </div>

          <div
            v-for="task in tasks"
            :key="task.id"
            class="task-card"
            :class="{
              'is-active': task.id === selectedId,
              'has-error': task.status === 'error',
            }"
            @click="selectedId = task.id"
          >
            <!-- 格式徽标 -->
            <div
              class="format-badge"
              :style="{ backgroundColor: getFormatCategory(task.extension).color + '22', color: getFormatCategory(task.extension).color }"
            >
              <component :is="getFormatCategory(task.extension).icon" :size="14" />
              <span>{{ getFormatCategory(task.extension).label }}</span>
            </div>

            <div class="task-info">
              <div class="task-title-row">
                <span class="task-name" :title="task.file_path">{{ task.file_name }}</span>
              </div>
              <div class="task-meta">
                <span v-if="task.file_size > 0">{{ formatSize(task.file_size) }}</span>
                <span v-if="task.duration_ms > 0">· {{ task.duration_ms }}ms</span>
                <span v-if="task.engine && task.engine !== 'pending'">· {{ task.engine }}</span>
              </div>
            </div>

            <!-- 状态标识与独立动作 -->
            <div class="task-actions">
              <div v-if="task.status === 'converting'" class="spinner"></div>
              <CheckCircle2 v-else-if="task.status === 'success'" class="icon-success" :size="18" />
              <AlertCircle v-else-if="task.status === 'error'" class="icon-error" :size="18" />
              <button
                v-else
                class="btn-icon"
                title="立即转换"
                @click.stop="convertSingleTask(task)"
              >
                <Play :size="14" />
              </button>

              <button
                class="btn-icon btn-remove"
                title="移除"
                @click="removeTask(task.id, $event)"
              >
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- 右侧：实时预览与导出控制台 -->
      <section class="preview-panel">
        <template v-if="selectedTask">
          <!-- 预览顶栏 -->
          <div class="preview-header">
            <div class="preview-file-meta">
              <h2>{{ selectedTask.file_name }}</h2>
              <span class="preview-path" :title="selectedTask.file_path">{{ selectedTask.file_path }}</span>
            </div>

            <div class="preview-toolbar">
              <!-- 视图切换 -->
              <div class="view-switch">
                <button
                  :class="{ active: viewMode === 'rendered' }"
                  @click="viewMode = 'rendered'"
                >
                  <Eye :size="14" /> 渲染视图
                </button>
                <button
                  :class="{ active: viewMode === 'raw' }"
                  @click="viewMode = 'raw'"
                >
                  <Code :size="14" /> 源码
                </button>
              </div>

              <button
                class="btn btn-secondary"
                :disabled="!selectedTask.markdown"
                @click="copyMarkdown(selectedTask.markdown)"
              >
                <Copy :size="14" /> 复制
              </button>

              <button
                class="btn btn-secondary"
                :disabled="!selectedTask.markdown"
                @click="handleSaveSingle(selectedTask)"
              >
                <Download :size="14" /> 另存为
              </button>
            </div>
          </div>

          <!-- 批量操作底栏/快捷栏 -->
          <div class="batch-export-bar">
            <span>批量导出能力：</span>
            <button class="btn-text" @click="handleSaveBatch">
              <FolderDown :size="14" /> 导出全部至文件夹
            </button>
            <span class="bar-divider">|</span>
            <button class="btn-text" @click="handleExportCombined">
              <Layers :size="14" /> 合并导出为单个 MD
            </button>
          </div>

          <!-- 内容展示区 -->
          <div class="preview-content-box">
            <!-- 错误情况 -->
            <div v-if="selectedTask.status === 'error'" class="error-notice">
              <AlertCircle :size="24" class="text-red" />
              <div>
                <strong>转换遇到问题</strong>
                <p>{{ selectedTask.error_message || "未知错误" }}</p>
              </div>
            </div>

            <!-- 进行中 -->
            <div v-else-if="selectedTask.status === 'converting'" class="loading-notice">
              <div class="spinner large"></div>
              <p>anydoc 正在加速转换中...</p>
            </div>

            <!-- 等待转换 -->
            <div v-else-if="selectedTask.status === 'idle'" class="idle-notice">
              <Play :size="32" class="text-sub" />
              <p>该文件尚未转换，点击右上角或列表中的开始按钮进行转换</p>
              <button class="btn btn-primary" @click="convertSingleTask(selectedTask)">
                立即转换此文件
              </button>
            </div>

            <!-- 渲染结果 -->
            <div
              v-else-if="viewMode === 'rendered'"
              class="markdown-body rendered-container"
              v-html="renderedHtml"
            ></div>

            <!-- 原始 Markdown 代码 -->
            <div v-else class="raw-code-container">
              <pre><code>{{ selectedTask.markdown }}</code></pre>
            </div>
          </div>
        </template>

        <!-- 未选择文件时的引导卡片 -->
        <div v-else class="preview-placeholder">
          <div class="placeholder-card">
            <div class="placeholder-icon">
              <Sparkles :size="40" />
            </div>
            <h2>欢迎使用 AnyDoc 转换器</h2>
            <p>从左侧拖入或选择您的任何办公文档、PDF、表格或代码文件，即刻获得干净利落的 GitHub Flavored Markdown。</p>

            <div class="format-grid">
              <div class="format-item">
                <strong>Office 核心</strong>
                <span>.docx, .pptx, .xlsx</span>
              </div>
              <div class="format-item">
                <strong>电子书与版式</strong>
                <span>.pdf, .epub, .rtf</span>
              </div>
              <div class="format-item">
                <strong>结构化数据</strong>
                <span>.csv, .odt, .ods</span>
              </div>
              <div class="format-item">
                <strong>代码与纯文本</strong>
                <span>.rs, .py, .js, .json, .txt</span>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>

    <!-- 浮层 Toast 通知 -->
    <transition name="fade">
      <div v-if="toastMessage" class="toast-box">
        {{ toastMessage }}
      </div>
    </transition>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  background-color: #090d16;
  color: #f1f5f9;
}

/* 顶部导航栏 */
.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background-color: #0f172a;
  border-bottom: 1px solid #1e293b;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-box {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(135deg, #38bdf8, #6366f1);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(56, 189, 248, 0.3);
}

.logo-icon {
  color: #ffffff;
  width: 22px;
  height: 22px;
}

.app-title {
  font-size: 16px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 8px;
  letter-spacing: -0.3px;
}

.badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  background-color: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.app-subtitle {
  font-size: 12px;
  color: #64748b;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.stats-capsule {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: #1e293b;
  border: 1px solid #334155;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 12px;
}

.stat-item {
  color: #94a3b8;
  display: flex;
  align-items: center;
  gap: 4px;
}

.stat-item b {
  color: #f8fafc;
}

.stat-divider {
  color: #475569;
}

.text-green { color: #34d399 !important; }
.text-red { color: #f87171 !important; }
.text-muted { color: #64748b !important; }

/* 按钮通用 */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
}

.btn-primary {
  background: linear-gradient(135deg, #38bdf8, #2563eb);
  color: #ffffff;
  box-shadow: 0 2px 6px rgba(37, 99, 235, 0.3);
}
.btn-primary:hover:not(:disabled) {
  opacity: 0.95;
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: #1e293b;
  border: 1px solid #334155;
  color: #e2e8f0;
}
.btn-secondary:hover:not(:disabled) {
  background-color: #334155;
}

.btn-danger-icon {
  padding: 8px;
  border-radius: 8px;
  background-color: rgba(244, 63, 94, 0.1);
  color: #f43f5e;
  border: 1px solid rgba(244, 63, 94, 0.2);
}
.btn-danger-icon:hover {
  background-color: rgba(244, 63, 94, 0.2);
}

/* 主工作区 */
.app-main {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* 左侧栏 */
.sidebar-panel {
  width: 380px;
  min-width: 320px;
  background-color: #0d1322;
  border-right: 1px solid #1e293b;
  display: flex;
  flex-direction: column;
}

.drop-zone {
  margin: 14px;
  padding: 16px;
  border: 2px dashed #334155;
  border-radius: 12px;
  background-color: rgba(30, 41, 59, 0.3);
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.drop-zone:hover, .drop-zone.is-dragging {
  border-color: #38bdf8;
  background-color: rgba(56, 189, 248, 0.08);
}

.drop-icon {
  color: #38bdf8;
}

.drop-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.drop-text strong {
  font-size: 13px;
  color: #f8fafc;
}

.drop-text span {
  font-size: 11px;
  color: #64748b;
}

.tasks-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 14px 14px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 220px;
  color: #64748b;
  font-size: 13px;
  text-align: center;
  gap: 6px;
}

.empty-list span {
  font-size: 11px;
  color: #475569;
}

.task-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  background-color: #131c2e;
  border: 1px solid #1e293b;
  cursor: pointer;
  transition: all 0.15s ease;
}

.task-card:hover {
  background-color: #1a253b;
  border-color: #334155;
}

.task-card.is-active {
  background-color: #1e293b;
  border-color: #38bdf8;
  box-shadow: 0 0 12px rgba(56, 189, 248, 0.15);
}

.task-card.has-error {
  border-color: rgba(248, 113, 113, 0.4);
}

.format-badge {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: 8px;
  font-size: 9px;
  font-weight: 700;
  gap: 2px;
  flex-shrink: 0;
}

.task-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.task-title-row {
  display: flex;
  align-items: center;
}

.task-name {
  font-size: 13px;
  font-weight: 500;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-meta {
  font-size: 11px;
  color: #64748b;
}

.task-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.icon-success { color: #34d399; }
.icon-error { color: #f87171; }

.btn-icon {
  padding: 6px;
  border-radius: 6px;
  color: #94a3b8;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-icon:hover {
  background-color: #334155;
  color: #ffffff;
}

.btn-remove:hover {
  color: #f43f5e;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(56, 189, 248, 0.2);
  border-top-color: #38bdf8;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
.spinner.large {
  width: 32px;
  height: 32px;
  border-width: 3px;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 右侧预览栏 */
.preview-panel {
  flex: 1;
  background-color: #0b1120;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-header {
  padding: 14px 20px;
  background-color: #0f172a;
  border-bottom: 1px solid #1e293b;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.preview-file-meta {
  min-width: 0;
}

.preview-file-meta h2 {
  font-size: 15px;
  font-weight: 600;
  color: #f8fafc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preview-path {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.preview-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.view-switch {
  display: flex;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 2px;
}

.view-switch button {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 12px;
  color: #94a3b8;
}

.view-switch button.active {
  background-color: #334155;
  color: #ffffff;
  font-weight: 500;
}

.batch-export-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 20px;
  background-color: rgba(30, 41, 59, 0.4);
  border-bottom: 1px solid #1e293b;
  font-size: 12px;
  color: #94a3b8;
}

.btn-text {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #38bdf8;
  font-size: 12px;
}
.btn-text:hover {
  text-decoration: underline;
}

.bar-divider {
  color: #334155;
}

.preview-content-box {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.rendered-container {
  max-width: 900px;
  margin: 0 auto;
}

.raw-code-container pre {
  margin: 0;
  padding: 16px;
  background-color: #070d17;
  border: 1px solid #1e293b;
  border-radius: 8px;
  color: #cbd5e1;
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.5;
  user-select: text;
  white-space: pre-wrap;
  word-break: break-all;
}

.error-notice, .loading-notice, .idle-notice {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  text-align: center;
  gap: 12px;
  color: #94a3b8;
}

.error-notice {
  background-color: rgba(248, 113, 113, 0.05);
  border: 1px dashed rgba(248, 113, 113, 0.2);
  border-radius: 12px;
  padding: 30px;
  margin: 20px;
}
.error-notice strong { color: #f87171; font-size: 15px; }

/* 引导占位卡片 */
.preview-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 40px;
}

.placeholder-card {
  max-width: 540px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.placeholder-icon {
  width: 72px;
  height: 72px;
  border-radius: 20px;
  background: linear-gradient(135deg, rgba(56, 189, 248, 0.2), rgba(99, 102, 241, 0.2));
  border: 1px solid rgba(56, 189, 248, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #38bdf8;
}

.placeholder-card h2 {
  font-size: 20px;
  font-weight: 700;
  color: #f8fafc;
}

.placeholder-card p {
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.6;
}

.format-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  width: 100%;
  margin-top: 12px;
  text-align: left;
}

.format-item {
  background-color: #131c2e;
  border: 1px solid #1e293b;
  padding: 12px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.format-item strong {
  font-size: 13px;
  color: #e2e8f0;
}

.format-item span {
  font-size: 11px;
  color: #64748b;
  font-family: var(--font-mono);
}

/* Toast */
.toast-box {
  position: fixed;
  bottom: 24px;
  right: 24px;
  background-color: #38bdf8;
  color: #041226;
  font-weight: 600;
  font-size: 13px;
  padding: 10px 18px;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(56, 189, 248, 0.3);
  z-index: 1000;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
