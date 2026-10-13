pub mod converter;

use converter::{BatchConvertSummary, ConvertResult, Converter};
use std::fs;
use std::path::Path;

#[tauri::command]
fn convert_file(path: String) -> Result<ConvertResult, String> {
    Ok(Converter::convert_file(&path))
}

#[tauri::command]
fn convert_files_batch(paths: Vec<String>) -> Result<BatchConvertSummary, String> {
    Ok(Converter::convert_batch(&paths))
}

#[tauri::command]
fn save_markdown(target_path: String, content: String) -> Result<String, String> {
    fs::write(&target_path, content).map_err(|e| format!("保存文件失败: {}", e))?;
    Ok(target_path)
}

#[tauri::command]
fn save_batch_markdown(target_dir: String, results: Vec<ConvertResult>) -> Result<usize, String> {
    let dir = Path::new(&target_dir);
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    let mut saved_count = 0;
    for item in results {
        if !item.success || item.markdown.is_empty() {
            continue;
        }

        let base_name = Path::new(&item.file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let out_file_name = format!("{}.md", base_name);
        let out_path = dir.join(out_file_name);
        if fs::write(&out_path, &item.markdown).is_ok() {
            saved_count += 1;
        }
    }

    Ok(saved_count)
}

#[tauri::command]
fn export_combined_markdown(target_path: String, results: Vec<ConvertResult>) -> Result<String, String> {
    let mut combined = String::new();
    for (idx, item) in results.iter().enumerate() {
        if !item.success || item.markdown.is_empty() {
            continue;
        }

        if idx > 0 {
            combined.push_str("\n\n---\n\n");
        }
        combined.push_str(&format!("# {}\n\n", item.file_name));
        combined.push_str(&item.markdown);
    }

    fs::write(&target_path, combined).map_err(|e| format!("合并导出失败: {}", e))?;
    Ok(target_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            convert_file,
            convert_files_batch,
            save_markdown,
            save_batch_markdown,
            export_combined_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
