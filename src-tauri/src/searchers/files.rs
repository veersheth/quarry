use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
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

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

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

    /// Score `path` against a multi-word query.
    ///
    /// Every word must match somewhere; returns None if any word has no match.
    /// Filename matches score 3× path-only matches so "readme" finds README.md
    /// before a deeply-nested file that has "readme" in a parent folder name.
    fn score(query_words: &[&str], path: &Path) -> Option<i64> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let path_str = path.to_string_lossy();

        let mut total = 0i64;
        for word in query_words {
            let name_score = MATCHER.fuzzy_match(&name, word).map(|s| s * 3);
            let path_score = MATCHER.fuzzy_match(&path_str, word);
            let word_score = match (name_score, path_score) {
                (None, None) => return None, // all words must match
                (a, b) => a.unwrap_or(0).max(b.unwrap_or(0)),
            };
            total += word_score;
        }
        Some(total)
    }

    fn collect_candidates(query_words: &[&str]) -> Vec<(PathBuf, i64)> {
        let mut hits: Vec<(PathBuf, i64)> = Vec::new();
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
                if let Some(score) = Self::score(query_words, &path) {
                    seen.insert(path.clone());
                    hits.push((path, score));
                }
            }
        }

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
        ResultItem::new(name, vec![ActionData::OpenUrl {
            url: format!("file://{}", path_str),
        }])
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

        // Trailing slash → directory listing
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

        // Absolute / home-relative path that exists
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

        // "downloads/monke" → scope walk to that root, fuzzy on "monke"
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

        let candidates = if let Some(base) = search_base_hint {
            let mut hits: Vec<(PathBuf, i64)> = Vec::new();
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
                if let Some(score) = Self::score(&query_words, &path) {
                    seen.insert(path.clone());
                    hits.push((path, score));
                }
            }
            hits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| mtime_secs(&b.0).cmp(&mtime_secs(&a.0))));
            hits
        } else {
            Self::collect_candidates(&query_words)
        };

        let results: Vec<ResultItem> = candidates
            .into_iter()
            .take(MAX_RESULTS)
            .map(|(path, _)| Self::path_to_result(path))
            .collect();

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
