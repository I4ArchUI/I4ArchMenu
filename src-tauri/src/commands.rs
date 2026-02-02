use crate::search::{search_apps, search_files, SearchResult};
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

#[tauri::command]
pub fn get_gtk_theme() -> String {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            // Output is usually "'prefer-dark'" or "'default'" with quotes and newline
            stdout.trim().replace("'", "")
        }
        Err(_) => "default".to_string(), // Fallback
    }
}
