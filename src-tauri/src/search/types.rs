use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub item_type: String, // "file", "folder", "app"
    pub icon: String,
}

impl SearchResult {
    pub fn new(name: String, path: String, item_type: String, icon: String) -> Self {
        Self {
            name,
            path,
            item_type,
            icon,
        }
    }
}
