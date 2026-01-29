use super::types::SearchResult;
use walkdir::WalkDir;

pub fn search_files(query: String, max_results: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    
    if query.is_empty() {
        return results;
    }

    // Prioritize most commonly used directories
    let search_paths = vec![
        dirs::home_dir().unwrap_or_default(),
        dirs::desktop_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
    ];

    for base_path in search_paths {
        if !base_path.exists() {
            continue;
        }

        for entry in WalkDir::new(&base_path)
            .max_depth(2) // Reduced from 3 to 2 for faster search
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if results.len() >= max_results {
                break;
            }

            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if file_name.to_lowercase().contains(&query_lower) {
                let item_type = if path.is_dir() { "folder" } else { "file" };
                let icon = get_file_icon(path);

                results.push(SearchResult::new(
                    file_name.to_string(),
                    path.to_string_lossy().to_string(),
                    item_type.to_string(),
                    icon.to_string(),
                ));
            }
        }

        if results.len() >= max_results {
            break;
        }
    }

    // Sort: folders first, then files
    results.sort_by(|a, b| {
        match (&a.item_type[..], &b.item_type[..]) {
            ("folder", "file") => std::cmp::Ordering::Less,
            ("file", "folder") => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    results
}

fn get_file_icon(path: &std::path::Path) -> &'static str {
    if path.is_dir() {
        return "pi pi-folder";
    }
    
    match path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => "pi pi-file-pdf",
        Some("doc") | Some("docx") => "pi pi-file-word",
        Some("xls") | Some("xlsx") => "pi pi-file-excel",
        Some("txt") | Some("md") => "pi pi-file-edit",
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") => "pi pi-image",
        Some("mp4") | Some("avi") | Some("mkv") => "pi pi-video",
        Some("mp3") | Some("wav") | Some("flac") => "pi pi-volume-up",
        Some("zip") | Some("tar") | Some("gz") => "pi pi-box",
        _ => "pi pi-file",
    }
}
