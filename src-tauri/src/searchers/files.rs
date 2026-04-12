use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::AppHandle;
use walkdir::WalkDir;

const MAX_DEPTH: usize = 4;
const MAX_RESULTS: usize = 10;
const SKIP_DIRS: &[&str] = &[
    "node_modules", "__pycache__", ".git", ".svn", ".hg",
    "target", "build", "dist", ".gradle", ".mvn", ".idea",
    ".vscode", "venv", ".venv", "env", ".env", "vendor",
    "bin", "obj", ".cache", "__MACOSX",
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

    /// If the query looks like a real path (absolute, or starts with ~/ or ./),
    /// return it expanded. Returns None if it's a plain search query.
    fn resolve_explicit_path(query: &str) -> Option<PathBuf> {
        let expanded = if let Some(rest) = query.strip_prefix("~/") {
            dirs::home_dir()?.join(rest)
        } else if query.starts_with('/') || query.starts_with("./") {
            PathBuf::from(query)
        } else {
            return None;
        };
        Some(expanded)
    }

    /// List the contents of a directory, sorted by mtime descending.
    fn list_dir(dir: &Path) -> Vec<PathBuf> {
        let mut entries: Vec<(PathBuf, u64)> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| {
                let p = e.path();
                let m = mtime_secs(&p);
                (p, m)
            })
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().map(|(p, _)| p).collect()
    }

    /// Fuzzy score: how well does `query` match `target`?
    /// Returns Some(score) where higher = better match, None = no match.
    /// Supports multi-word queries — every word must match somewhere in the path.
    fn fuzzy_score(query_words: &[&str], path: &Path) -> Option<u32> {
        let path_lower = path.to_string_lossy().to_lowercase();
        let name_lower = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let mut total_score: u32 = 0;

        for word in query_words {
            // Each word must appear somewhere in the full path
            let pos_in_path = path_lower.find(word)?;
            let pos_in_name = name_lower.find(word);

            // Name matches score much higher than path-only matches
            let word_score = if let Some(pos) = pos_in_name {
                // Bonus for matching at the start of the filename
                if pos == 0 { 100 } else { 60 }
            } else {
                // Still valid — matched somewhere in the path
                let _ = pos_in_path; // already confirmed above
                20
            };

            // Bonus for exact substring match (not just fuzzy)
            let consecutive_bonus = if name_lower.contains(*word) { 20 } else { 0 };

            // Bonus for directories (they help narrow results naturally)
            let dir_bonus = if path.is_dir() { 10 } else { 0 };

            total_score += word_score + consecutive_bonus + dir_bonus;
        }

        Some(total_score)
    }

    fn collect_candidates(query_words: &[&str]) -> Vec<(PathBuf, u32)> {
        let mut hits: Vec<(PathBuf, u32)> = Vec::new();
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
                let path = entry.path().to_path_buf();
                if seen.contains(&path) {
                    continue;
                }
                if let Some(score) = Self::fuzzy_score(query_words, &path) {
                    seen.insert(path.clone());
                    hits.push((path, score));
                }
            }
        }

        // Sort by score desc, then mtime desc for ties
        hits.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| mtime_secs(&b.0).cmp(&mtime_secs(&a.0)))
        });

        hits
    }

    fn icon_for(path: &Path) -> &'static str {
        if path.is_dir() { "icons/folder.png" } else { "icons/file.png" }
    }

    fn path_to_result(path: PathBuf) -> ResultItem {
        let path_str = path.to_string_lossy().into_owned();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();
        let icon = Self::icon_for(&path);
        ResultItem::new(name, ActionData::OpenUrl {
            url: format!("file://{}", path_str),
        })
        .description(path_str)
        .icon(icon)
    }
}

impl SearchProvider for FileSearcher {

    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return SearchResult { results: vec![], result_type: ResultType::List };
    }

    if trimmed.ends_with('/') {
        let path_str = trimmed.trim_end_matches('/');

        let dir = if let Some(p) = Self::resolve_explicit_path(path_str) {
            Some(p)
        } else {
            let lower = path_str.to_lowercase();
            Self::search_paths().into_iter().find(|base| {
                base.file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default()
                    == lower
            })
        };

        if let Some(dir) = dir.filter(|d| d.is_dir()) {
            let mut results: Vec<ResultItem> = Self::list_dir(&dir)
                .into_iter()
                .map(Self::path_to_result)
                .collect();
            results.truncate(MAX_RESULTS);
            return SearchResult { results, result_type: ResultType::List };
        }
    }

    if let Some(explicit) = Self::resolve_explicit_path(trimmed) {
        if explicit.exists() {
            if explicit.is_dir() {
                let mut results: Vec<ResultItem> = Self::list_dir(&explicit)
                    .into_iter()
                    .map(Self::path_to_result)
                    .collect();
                results.truncate(MAX_RESULTS);
                return SearchResult { results, result_type: ResultType::List };
            } else {
                return SearchResult {
                    results: vec![Self::path_to_result(explicit)],
                    result_type: ResultType::List,
                };
            }
        }
    }

    let (search_base_hint, fuzzy_query) = if let Some(sep) = trimmed.rfind('/') {
        let dir_part = &trimmed[..sep];
        let name_part = trimmed[sep + 1..].trim();

        if name_part.is_empty() {
            (None, trimmed) 
        } else {
            let lower = dir_part.to_lowercase();
            let hint = Self::search_paths().into_iter().find(|base| {
                let base_name = base
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                base_name == lower || base_name.contains(&lower)
            });
            (hint, name_part)
        }
    } else {
        (None, trimmed)
    };

    let query_words: Vec<&str> = fuzzy_query.split_whitespace().collect();
    if query_words.is_empty() {
        return SearchResult { results: vec![], result_type: ResultType::List };
    }

    // // ... rest unchanged
    //
    // fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
    //     let trimmed = query.trim();
    //     if trimmed.is_empty() {
    //         return SearchResult { results: vec![], result_type: ResultType::List };
    //     }
    //
    //     // ── 1. Explicit path with trailing slash → directory listing ──────────
    //     let is_dir_browse = trimmed.ends_with('/');
    //     if is_dir_browse {
    //         let path_str = trimmed.trim_end_matches('/');
    //         let dir = if let Some(p) = Self::resolve_explicit_path(path_str) {
    //             p
    //         } else {
    //             PathBuf::from(path_str)
    //         };
    //
    //         if dir.is_dir() {
    //             let mut results: Vec<ResultItem> = Self::list_dir(&dir)
    //                 .into_iter()
    //                 .map(Self::path_to_result)
    //                 .collect();
    //             results.truncate(MAX_RESULTS);
    //             return SearchResult { results, result_type: ResultType::List };
    //         }
    //     }
    //
    //     // ── 2. Explicit absolute/home path (no trailing slash) ────────────────
    //     if let Some(explicit) = Self::resolve_explicit_path(trimmed) {
    //         if explicit.exists() {
    //             if explicit.is_dir() {
    //                 // Treat like a directory browse
    //                 let mut results: Vec<ResultItem> = Self::list_dir(&explicit)
    //                     .into_iter()
    //                     .map(Self::path_to_result)
    //                     .collect();
    //                 results.truncate(MAX_RESULTS);
    //                 return SearchResult { results, result_type: ResultType::List };
    //             } else {
    //                 return SearchResult {
    //                     results: vec![Self::path_to_result(explicit)],
    //                     result_type: ResultType::List,
    //                 };
    //             }
    //         }
    //     }
    //
    //     // ── 3. Query contains a path separator → treat prefix as dir hint ─────
    //     // e.g. "downloads/monke" → search only under ~/Downloads, fuzzy on "monke"
    //     let (search_base_hint, fuzzy_query) = if let Some(sep) = trimmed.rfind('/') {
    //         let dir_part = &trimmed[..sep];
    //         let name_part = trimmed[sep + 1..].trim();
    //
    //         // Try to resolve the directory part against known search roots
    //         let hint = Self::search_paths().into_iter().find(|base| {
    //             let base_name = base
    //                 .file_name()
    //                 .map(|n| n.to_string_lossy().to_lowercase())
    //                 .unwrap_or_default();
    //             base_name.contains(&dir_part.to_lowercase())
    //                 || base.to_string_lossy().to_lowercase().ends_with(&dir_part.to_lowercase())
    //         });
    //
    //         (hint, name_part)
    //     } else {
    //         (None, trimmed)
    //     };
    //
    //     // ── 4. Fuzzy search ───────────────────────────────────────────────────
    //     let query_words: Vec<&str> = fuzzy_query.split_whitespace().collect();
    //     if query_words.is_empty() {
    //         return SearchResult { results: vec![], result_type: ResultType::List };
    //     }
    //
        let candidates = if let Some(base) = search_base_hint {
            // Scoped walk under the hinted directory
            let mut hits: Vec<(PathBuf, u32)> = Vec::new();
            let mut seen = std::collections::HashSet::new();

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
                let path = entry.path().to_path_buf();
                if seen.contains(&path) { continue; }
                if let Some(score) = Self::fuzzy_score(&query_words, &path) {
                    seen.insert(path.clone());
                    hits.push((path, score));
                }
            }
            hits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| mtime_secs(&b.0).cmp(&mtime_secs(&a.0))));
            hits
        } else {
            Self::collect_candidates(&query_words)
        };

        let mut results: Vec<ResultItem> = candidates
            .into_iter()
            .map(|(path, _score)| Self::path_to_result(path))
            .collect();

        // Run through the trait's fuzzy_filter for final ranking polish
        results = self.fuzzy_filter(results, fuzzy_query);
        results.truncate(MAX_RESULTS);

        SearchResult { results, result_type: ResultType::List }
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
