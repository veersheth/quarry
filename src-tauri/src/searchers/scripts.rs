use std::path::PathBuf;
use tauri::AppHandle;

use super::SearchProvider;
use super::files::SCRIPT_EXTENSIONS;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct ScriptsSearcher;

fn scripts_dir() -> Option<PathBuf> {
    let configured = crate::CONFIG.read().ok()?.scripts.path.clone();
    let dir = if let Some(rest) = configured.strip_prefix("~/") {
        dirs::home_dir()?.join(rest)
    } else {
        PathBuf::from(&configured)
    };
    if dir.exists() { Some(dir) } else { None }
}

fn is_runnable(path: &std::path::Path) -> bool {
    if path.is_dir() { return false; }
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if SCRIPT_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
            return true;
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.permissions().mode() & 0o111 != 0 {
                return true;
            }
        }
    }
    false
}

fn build_item(path: &std::path::Path, scripts_dir: &std::path::Path) -> ResultItem {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let rel = path.strip_prefix(scripts_dir)
        .map(|r| r.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());
    let path_str = path.to_string_lossy().into_owned();

    ResultItem::new(name, vec![
        Action::new("Run", ActionData::RunScript { path: path_str.clone() }),
        Action::new("Open", ActionData::OpenUrl { url: format!("file://{}", path_str) }),
        Action::new("Open Containing Folder", ActionData::OpenUrl {
            url: format!("file://{}", scripts_dir.to_string_lossy()),
        }),
        Action::new("Copy Path", ActionData::CopyToClipboard { text: path_str }),
    ])
    .description(rel)
    .icon("icons/file.png")
}

impl SearchProvider for ScriptsSearcher {
    fn name(&self) -> String { "scripts".to_string() }

    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim().to_lowercase();

        let Some(dir) = scripts_dir() else {
            return SearchResult {
                results: vec![ResultItem::new(
                    "Scripts folder not found",
                    vec![],
                ).description(
                    crate::CONFIG.read().ok()
                        .map(|c| c.scripts.path.clone())
                        .unwrap_or_default()
                )],
                result_type: ResultType::List,
                ..Default::default()
            };
        };

        // Collect all runnable files, walking one level of subdirectories
        let mut scripts: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // One level deep
                    if let Ok(sub) = std::fs::read_dir(&path) {
                        for sub_entry in sub.flatten() {
                            let sp = sub_entry.path();
                            if is_runnable(&sp) {
                                scripts.push(sp);
                            }
                        }
                    }
                } else if is_runnable(&path) {
                    scripts.push(path);
                }
            }
        }

        scripts.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        let items: Vec<ResultItem> = if query.is_empty() {
            scripts.iter().map(|p| build_item(p, &dir)).collect()
        } else {
            let mut scored: Vec<(PathBuf, i64)> = scripts.into_iter().filter_map(|p| {
                let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                crate::search_utils::smart_match(&name, &query).map(|s| (p, s))
            }).collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            scored.iter().map(|(p, _)| build_item(p, &dir)).collect()
        };

        if items.is_empty() && !query.is_empty() {
            return SearchResult {
                results: vec![ResultItem::new(
                    format!("No scripts matching \"{}\"", query),
                    vec![],
                )],
                result_type: ResultType::List,
                ..Default::default()
            };
        }

        SearchResult {
            results: items,
            result_type: ResultType::List,
            ..Default::default()
        }
    }
}
