use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::ACTION_REGISTRY;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::AppHandle;
use walkdir::WalkDir;

// ---------------------------------------------------------
// CONSTANTS
// ---------------------------------------------------------

const MAX_DEPTH: usize = 4;
const MAX_RESULTS: usize = 10;

/// Directories that are never worth descending into.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "__pycache__",
    ".git",
    ".svn",
    ".hg",
    "target",
    "build",
    "dist",
    ".gradle",
    ".mvn",
    ".idea",
    ".vscode",
    "venv",
    ".venv",
    "env",
    ".env",
    "vendor",
    "bin",
    "obj",
    ".cache",
    "__MACOSX",
];

// ---------------------------------------------------------
// SEARCHER
// ---------------------------------------------------------

pub struct FileSearcher;

impl FileSearcher {
    /// Directories to walk. Add more arms here to expand search scope.
    fn search_paths() -> Vec<PathBuf> {
        [
            dirs::home_dir(),
            // dirs::document_dir(),
            // dirs::download_dir(),
            // dirs::desktop_dir(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// Walk `search_paths`, collecting files whose names contain `query`,
    /// sorted by modification time (newest first).
    fn search_files(query: &str) -> Vec<PathBuf> {
        let query_lower = query.to_lowercase();
        let mut hits: Vec<(PathBuf, u64)> = Vec::new();

        'outer: for base in Self::search_paths() {
            for entry in WalkDir::new(&base)
                .max_depth(MAX_DEPTH)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_str().unwrap_or("");
                    // skip hidden files/dirs and known bloat dirs
                    !name.starts_with('.') && !(e.path().is_dir() && SKIP_DIRS.contains(&name))
                })
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if name.to_lowercase().contains(&query_lower) {
                    let mtime = mtime_secs(path);
                    hits.push((path.to_path_buf(), mtime));

                    if hits.len() >= MAX_RESULTS * 3 {
                        break 'outer;
                    }
                }
            }
        }

        hits.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        hits.truncate(MAX_RESULTS);
        hits.into_iter().map(|(p, _)| p).collect()
    }

    /// Returns an icon path based on whether the path is a directory or file.
    fn icon_for(path: &Path) -> String {
        if path.is_dir() {
            "icons/folder.png".to_string()
        } else {
            "icons/file.png".to_string()
        }
    }

    /// Stable, collision-resistant action ID for a path.
    fn action_id(path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("file_{:x}", hasher.finish())
    }
}

impl SearchProvider for FileSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
            };
        }

        let results = Self::search_files(trimmed)
            .into_iter()
            .filter_map(|path| {
                let path_str = path.to_string_lossy().into_owned();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path_str)
                    .to_string();

                let action_id = Self::action_id(&path);

                ACTION_REGISTRY
                    .lock()
                    .ok()?
                    .register(
                        action_id.clone(),
                        ActionData::OpenUrl {
                            url: format!("file://{}", path_str),
                        },
                    );

                Some(ResultItem {
                    name,
                    action_id,
                    description: Some(path_str),
                    icon: Some(Self::icon_for(&path)),
                })
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}

fn mtime_secs(path: &Path) -> u64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
