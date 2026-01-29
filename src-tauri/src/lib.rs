mod search;
mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search_files_command,
            search_apps_command,
            open_item,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

