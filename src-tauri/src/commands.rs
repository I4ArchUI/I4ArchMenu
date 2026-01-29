use crate::search::{SearchResult, search_files, search_apps};
use std::path::PathBuf;

#[tauri::command]
pub fn search_files_command(query: String, max_results: usize) -> Vec<SearchResult> {
    search_files(query, max_results)
}

#[tauri::command]
pub fn search_apps_command(query: String, max_results: usize) -> Vec<SearchResult> {
    search_apps(query, max_results)
}

#[tauri::command]
pub fn open_item(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    
    if path.ends_with(".desktop") {
        // Launch application from .desktop file
        std::process::Command::new("gtk-launch")
            .arg(path_buf.file_stem().unwrap_or_default())
            .spawn()
            .map_err(|e| e.to_string())?;
    } else {
        // Open file/folder with default application
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn exit_app() {
    std::process::exit(0);
}
