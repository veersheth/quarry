use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::AppHandle;
use walkdir::WalkDir;

const MAX_DEPTH: usize = 4;
const MAX_RESULTS: usize = 10;

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

pub struct FileSearcher;

impl FileSearcher {
    fn search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for dir in [
            dirs::download_dir(),
            dirs::document_dir(),
            dirs::desktop_dir(),
        ] {
            if let Some(p) = dir {
                paths.push(p);
            }
        }

        if let Some(home) = dirs::home_dir() {
            paths.push(home);
        }

        paths.dedup();
        paths
    }

    fn collect_candidates(query: &str) -> Vec<PathBuf> {
        let query_lower = query.to_lowercase();
        let mut hits: Vec<PathBuf> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for base in Self::search_paths() {
            for entry in WalkDir::new(&base)
                .max_depth(MAX_DEPTH)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_str().unwrap_or("");
                    !name.starts_with('.') && !(e.path().is_dir() && SKIP_DIRS.contains(&name))
                })
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                let path_str = path.to_string_lossy().to_lowercase();
                let all_words_match = query_lower
                    .split_whitespace()
                    .all(|word| path_str.contains(word));

                if all_words_match {
                    let owned = path.to_path_buf();
                    if seen.insert(owned.clone()) {
                        hits.push(owned);
                    }
                }
            }
        }

        hits
    }

    fn icon_for(path: &Path) -> &'static str {
        if path.is_dir() { "icons/folder.png" } else { "icons/file.png" }
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

        let candidates: Vec<ResultItem> = Self::collect_candidates(trimmed)
            .into_iter()
            .map(|path| {
                let path_str = path.to_string_lossy().into_owned();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path_str)
                    .to_string();

                ResultItem::new(name, ActionData::OpenUrl {
                    url: format!("file://{}", path_str),
                })
                .description(path_str)
                .icon(Self::icon_for(&path))
            })
            .collect();

        let mut results = self.fuzzy_filter(candidates, trimmed);
        results.truncate(MAX_RESULTS);

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
