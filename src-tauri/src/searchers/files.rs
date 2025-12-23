use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::time::SystemTime;

pub struct FileSearcher;

impl FileSearcher {
    fn get_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            paths.push(home);
        }
        
        // if let Some(documents) = dirs::document_dir() {
        //     paths.push(documents);
        // }
        // if let Some(downloads) = dirs::download_dir() {
        //     paths.push(downloads);
        // }
        // if let Some(desktop) = dirs::desktop_dir() {
        //     paths.push(desktop);
        // }
        
        paths
    }

    fn search_files(query: &str, max_results: usize) -> Vec<(PathBuf, u64)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for base_path in Self::get_search_paths() {
            if results.len() >= max_results * 2 {
                break;
            }
            
            for entry in WalkDir::new(&base_path)
                .max_depth(4)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    let file_name = e.file_name().to_str().unwrap_or("");
                    
                    if file_name.starts_with('.') { // hidden
                        return false;
                    }
                    
                    let skip_dirs = [ // bloat
                        "node_modules",
                        "__pycache__",
                        ".git",
                        ".svn",
                        ".hg",
                        "target",      // Rust build directory
                        "build",       // Common build directory
                        "dist",        // Distribution directory
                        ".gradle",     // Gradle cache
                        ".mvn",        // Maven directory
                        ".idea",       // IntelliJ IDEA
                        ".vscode",     // VS Code settings
                        "venv",        // Python virtual environment
                        ".venv",       // Python virtual environment
                        "env",         // Environment directory
                        ".env",        // Environment directory
                        "vendor",      // PHP/Ruby dependencies
                        "bin",         // Binary directory (in some contexts)
                        "obj",         // Object files
                        ".cache",      // Cache directory
                        "__MACOSX",    // macOS metadata
                        ".DS_Store",   // macOS file
                        "Thumbs.db",   // Windows thumbnail cache
                    ];
                    
                    if e.path().is_dir() && skip_dirs.contains(&file_name) {
                        return false;
                    }
                    
                    true
                })
                .filter_map(|e| e.ok())
            {
                if results.len() >= max_results * 2 {
                    break;
                }
                
                let path = entry.path();
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if file_name.to_lowercase().contains(&query_lower) {
                    let modified = std::fs::metadata(path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    
                    results.push((path.to_path_buf(), modified));
                }
            }
        }
        
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(max_results);
        
        results
    }

    fn get_file_icon(path: &PathBuf) -> String {
        if path.is_dir() {
            "icons/folder.png".to_string()
        } else {
            "icons/file.png".to_string()
        }
    }
}

impl SearchProvider for FileSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let trimmed = query.trim();
        
        if trimmed.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
                usage_sorted: true,
                additional_info: None,
            };
        }
        
        let files = Self::search_files(trimmed, 10);
        let mut results = Vec::new();
        
        for (path, _) in files {
            let path_str = path.to_string_lossy().to_string();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            let action_id = format!("file_{}", path_str.replace('/', "_").replace(' ', "_"));
            
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::OpenUrl {
                        url: format!("file://{}", path_str),
                    },
                );
            }
            
            let icon = Some(Self::get_file_icon(&path));
            
            results.push(ResultItem {
                name: file_name,
                action_id,
                description: Some(path_str),
                icon,
            });
        }
        
        SearchResult {
            results,
            result_type: ResultType::List,
            usage_sorted: true,
            additional_info: None,
        }
    }
}
