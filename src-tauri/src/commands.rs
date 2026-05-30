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
        let file_stem = path_buf.file_stem().unwrap_or_default().to_string_lossy().to_string();

        // 1. Try launching Flatpak apps directly via 'flatpak run'
        if path.contains("flatpak") {
            if std::process::Command::new("flatpak")
                .args(["run", &file_stem])
                .spawn()
                .is_ok() {
                return Ok(());
            }
        }

        // 2. Parse the desktop file content to run the command directly
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(exec_line) = parse_exec_command(&content) {
                let clean_cmd = clean_exec_line(&exec_line);
                let parts: Vec<&str> = clean_cmd.split_whitespace().collect();
                if !parts.is_empty() {
                    let mut cmd = std::process::Command::new(parts[0]);
                    if parts.len() > 1 {
                        cmd.args(&parts[1..]);
                    }
                    if cmd.spawn().is_ok() {
                        return Ok(());
                    }
                }
            }
        }

        // 3. Fallback to standard gtk-launch
        std::process::Command::new("gtk-launch")
            .arg(&file_stem)
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

fn parse_exec_command(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("Exec=") {
            return Some(line.trim_start_matches("Exec=").to_string());
        }
    }
    None
}

fn clean_exec_line(exec: &str) -> String {
    let mut clean = exec.to_string();
    for placeholder in &["%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v"] {
        clean = clean.replace(placeholder, "");
    }
    clean.trim().to_string()
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
