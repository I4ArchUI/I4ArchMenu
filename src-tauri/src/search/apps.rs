use super::types::SearchResult;
use std::path::PathBuf;

pub fn search_apps(query: String, max_results: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    if query.is_empty() {
        return results;
    }

    // Search for .desktop files in common locations, including system and Flatpak paths
    let app_paths = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/share/flatpak/exports/share/applications"),
    ];

    'outer: for app_path in app_paths {
        if !app_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&app_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if results.len() >= max_results {
                    break 'outer;
                }

                let path = entry.path();
                
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(app_name) = parse_desktop_name(&content) {
                        if app_name.to_lowercase().contains(&query_lower) {
                            let path_str = path.to_string_lossy().to_string();
                            let is_flatpak = path_str.contains("flatpak");
                            let item_type = if is_flatpak { "flatpak".to_string() } else { "app".to_string() };
                            let icon = if is_flatpak { "pi pi-box".to_string() } else { "pi pi-desktop".to_string() };

                            // Deduplicate by name and type
                            if !results.iter().any(|r| r.name == app_name && r.item_type == item_type) {
                                results.push(SearchResult::new(
                                    app_name,
                                    path_str,
                                    item_type,
                                    icon,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort alphabetically
    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    results
}

fn parse_desktop_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("Name=") {
            return Some(line.trim_start_matches("Name=").to_string());
        }
    }
    None
}
