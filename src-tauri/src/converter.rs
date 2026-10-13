use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvertResult {
    pub file_path: String,
    pub file_name: String,
    pub extension: String,
    pub file_size: u64,
    pub success: bool,
    pub markdown: String,
    pub error_message: Option<String>,
    pub engine: String,
    pub duration_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchConvertSummary {
    pub total: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub results: Vec<ConvertResult>,
}

pub struct Converter;

impl Converter {
    /// 转换单个文件为 Markdown
    pub fn convert_file(path_str: &str) -> ConvertResult {
        let start = Instant::now();
        let path = Path::new(path_str);

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                return ConvertResult {
                    file_path: path_str.to_string(),
                    file_name,
                    extension,
                    file_size: 0,
                    success: false,
                    markdown: String::new(),
                    error_message: Some(format!("无法读取文件元信息: {}", e)),
                    engine: "none".to_string(),
                    duration_ms: start.elapsed().as_millis(),
                };
            }
        };

        let file_size = metadata.len();

        // 1. 首先尝试使用 anydoc 核心库进行文档转换
        match anydoc::to_markdown(path) {
            Ok(md) => ConvertResult {
                file_path: path_str.to_string(),
                file_name,
                extension,
                file_size,
                success: true,
                markdown: md,
                error_message: None,
                engine: "anydoc".to_string(),
                duration_ms: start.elapsed().as_millis(),
            },
            Err(err) => {
                match err {
                    anydoc::ConvertError::Unsupported(_) => {
                        Self::fallback_convert(path, &file_name, &extension, file_size, start)
                    }
                    anydoc::ConvertError::NeedsOcr { pages, .. } => ConvertResult {
                        file_path: path_str.to_string(),
                        file_name,
                        extension,
                        file_size,
                        success: false,
                        markdown: String::new(),
                        error_message: Some(format!(
                            "PDF 文件包含扫描页面（第 {:?} 页需 OCR），本地模式暂不支持纯图片文字识别",
                            pages
                        )),
                        engine: "anydoc".to_string(),
                        duration_ms: start.elapsed().as_millis(),
                    },
                    anydoc::ConvertError::Encrypted => ConvertResult {
                        file_path: path_str.to_string(),
                        file_name,
                        extension,
                        file_size,
                        success: false,
                        markdown: String::new(),
                        error_message: Some("文件已加密或设有密码保护，无法转换".to_string()),
                        engine: "anydoc".to_string(),
                        duration_ms: start.elapsed().as_millis(),
                    },
                    anydoc::ConvertError::Malformed { detail, .. } => ConvertResult {
                        file_path: path_str.to_string(),
                        file_name,
                        extension,
                        file_size,
                        success: false,
                        markdown: String::new(),
                        error_message: Some(format!("文件格式损坏或不符合规范: {}", detail)),
                        engine: "anydoc".to_string(),
                        duration_ms: start.elapsed().as_millis(),
                    },
                    _other => {
                        Self::fallback_convert(path, &file_name, &extension, file_size, start)
                    }
                }
            }
        }
    }

    /// 智能回退转换：针对代码文件、纯文本及二进制提供良好的 Markdown 表现
    fn fallback_convert(
        path: &Path,
        file_name: &str,
        extension: &str,
        file_size: u64,
        start: Instant,
    ) -> ConvertResult {
        let path_str = path.to_string_lossy().to_string();

        // 尝试以 UTF-8 文本读取
        match fs::read_to_string(path) {
            Ok(content) => {
                // 如果本身就是 markdown 格式
                if extension == "md" || extension == "markdown" {
                    return ConvertResult {
                        file_path: path_str,
                        file_name: file_name.to_string(),
                        extension: extension.to_string(),
                        file_size,
                        success: true,
                        markdown: content,
                        error_message: None,
                        engine: "raw-markdown".to_string(),
                        duration_ms: start.elapsed().as_millis(),
                    };
                }

                // 识别源码语言高亮
                let lang = match extension {
                    "rs" => "rust",
                    "js" | "mjs" | "cjs" => "javascript",
                    "ts" | "mts" | "cts" => "typescript",
                    "py" => "python",
                    "java" => "java",
                    "c" | "h" => "c",
                    "cpp" | "cc" | "cxx" | "hpp" => "cpp",
                    "go" => "go",
                    "html" | "htm" => "html",
                    "css" | "scss" | "less" => "css",
                    "json" => "json",
                    "yaml" | "yml" => "yaml",
                    "toml" => "toml",
                    "xml" | "svg" => "xml",
                    "sql" => "sql",
                    "sh" | "bash" | "zsh" => "bash",
                    "php" => "php",
                    "rb" => "ruby",
                    "swift" => "swift",
                    "kt" | "kts" => "kotlin",
                    "dockerfile" => "dockerfile",
                    _ => "",
                };

                let markdown = if !lang.is_empty() {
                    format!("### `{}`\n\n```{}\n{}\n```\n", file_name, lang, content)
                } else if extension == "txt" || extension == "log" || extension.is_empty() {
                    format!("### `{}`\n\n{}\n", file_name, content)
                } else {
                    format!("### `{}`\n\n```\n{}\n```\n", file_name, content)
                };

                ConvertResult {
                    file_path: path_str,
                    file_name: file_name.to_string(),
                    extension: extension.to_string(),
                    file_size,
                    success: true,
                    markdown,
                    error_message: None,
                    engine: "text-code-wrapper".to_string(),
                    duration_ms: start.elapsed().as_millis(),
                }
            }
            Err(_) => {
                // 读取非 UTF-8 文本，说明可能是二进制媒体文件（图片、音视频、zip 等）
                let markdown = format!(
                    "### `{}`\n\n> ⚠️ 该文件为非文本/媒体二进制文件，无法提取直接排版文本。\n\n| 属性 | 信息 |\n| --- | --- |\n| 文件名 | `{}` |\n| 格式后缀 | `{}` |\n| 文件大小 | `{}` 字节 |\n",
                    file_name, file_name, extension, file_size
                );

                ConvertResult {
                    file_path: path_str,
                    file_name: file_name.to_string(),
                    extension: extension.to_string(),
                    file_size,
                    success: true,
                    markdown,
                    error_message: None,
                    engine: "binary-metadata".to_string(),
                    duration_ms: start.elapsed().as_millis(),
                }
            }
        }
    }

    /// 批量转换多个文件路径（包含递归扫描文件夹）
    pub fn convert_batch(paths: &[String]) -> BatchConvertSummary {
        let mut all_files = Vec::new();

        for p_str in paths {
            let p = Path::new(p_str);
            if p.is_dir() {
                for entry in walkdir::WalkDir::new(p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                {
                    all_files.push(entry.path().to_string_lossy().to_string());
                }
            } else if p.is_file() {
                all_files.push(p_str.clone());
            }
        }

        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for file_path in &all_files {
            let res = Self::convert_file(file_path);
            if res.success {
                success_count += 1;
            } else {
                failure_count += 1;
            }
            results.push(res);
        }

        BatchConvertSummary {
            total: all_files.len(),
            success_count,
            failure_count,
            results,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_convert_markdown_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# 标题\n\n这是测试正文").unwrap();
        let path = file.path().to_str().unwrap();

        let result = Converter::convert_file(path);
        assert!(result.success);
        assert!(result.markdown.contains("这是测试正文"));
    }

    #[test]
    fn test_convert_code_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        fs::write(&file_path, "fn main() { println!(\"Hello\"); }").unwrap();

        let result = Converter::convert_file(file_path.to_str().unwrap());
        assert!(result.success);
        assert_eq!(result.engine, "text-code-wrapper");
        assert!(result.markdown.contains("```rust"));
        assert!(result.markdown.contains("fn main()"));
    }

    #[test]
    fn test_convert_csv_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.csv");
        fs::write(&file_path, "name,age,city\nAlice,30,Beijing\nBob,25,Shanghai\n").unwrap();

        let result = Converter::convert_file(file_path.to_str().unwrap());
        assert!(result.success);
        // anydoc 原生支持 csv 转 markdown 表格
        assert!(result.markdown.contains("Alice"));
        assert!(result.markdown.contains("Beijing"));
    }
}
