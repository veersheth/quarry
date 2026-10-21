use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, SearchResult};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use notify::event::EventKind;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tauri::AppHandle;
use walkdir::WalkDir;

const MAX_DEPTH: usize = 4;
const MAX_RESULTS: usize = 10;
// A rare recovery scan catches events lost to kernel queue overflow or unusual filesystems.
const FALLBACK_REFRESH_SECS: u64 = 3600;

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

pub(crate) const SCRIPT_EXTENSIONS: &[&str] = &[
    "sh", "bash", "zsh", "fish", "py", "rb", "js", "mjs", "ts", "pl", "pm", "lua", "ps1", "bat",
    "cmd", "r", "php",
];

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

// ---------------------------------------------------------------------------
// Background file index
// ---------------------------------------------------------------------------

type IndexedFile = (PathBuf, u64);
type FileIndex = Arc<Vec<IndexedFile>>;

static FILE_INDEX: Lazy<RwLock<FileIndex>> = Lazy::new(|| RwLock::new(Arc::new(Vec::new())));
static USER_SCRIPTS_DIR: Lazy<Option<PathBuf>> = Lazy::new(get_user_scripts_dir);

/// Build the index once, then maintain it from filesystem events.
pub fn start_file_index() {
    std::thread::spawn(|| {
        let roots = FileSearcher::search_paths();
        let (entries, directories) = build_index_snapshot(&roots);
        if let Ok(mut idx) = FILE_INDEX.write() {
            *idx = Arc::new(entries);
        }

        let (tx, rx) = std::sync::mpsc::sync_channel(4096);
        let overflowed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_overflowed = Arc::clone(&overflowed);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if should_refresh_for_event(&event) {
                    if tx.try_send(event).is_err() {
                        callback_overflowed.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        });

        let mut watcher = watcher.ok();
        let mut watched = std::collections::HashSet::new();
        if let Some(watcher) = watcher.as_mut() {
            watch_directories(watcher, directories, &mut watched);
        }

        loop {
            match rx.recv_timeout(Duration::from_secs(FALLBACK_REFRESH_SECS)) {
                Ok(first) => {
                    let mut events = vec![first];
                    while let Ok(event) = rx.recv_timeout(Duration::from_millis(150)) {
                        events.push(event);
                    }

                    if overflowed.swap(false, std::sync::atomic::Ordering::Relaxed) {
                        let (entries, directories) = build_index_snapshot(&roots);
                        replace_index(entries);
                        if let Some(watcher) = watcher.as_mut() {
                            reset_watches(watcher, directories, &mut watched);
                        }
                        continue;
                    }

                    for event in events {
                        apply_event(event, &roots, watcher.as_mut(), &mut watched);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let (entries, directories) = build_index_snapshot(&roots);
                    replace_index(entries);
                    if let Some(watcher) = watcher.as_mut() {
                        reset_watches(watcher, directories, &mut watched);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

fn watch_directories(
    watcher: &mut RecommendedWatcher,
    directories: impl IntoIterator<Item = PathBuf>,
    watched: &mut std::collections::HashSet<PathBuf>,
) {
    for path in directories {
        if watched.insert(path.clone())
            && watcher.watch(&path, RecursiveMode::NonRecursive).is_err()
        {
            watched.remove(&path);
        }
    }
}

fn reset_watches(
    watcher: &mut RecommendedWatcher,
    directories: Vec<PathBuf>,
    watched: &mut std::collections::HashSet<PathBuf>,
) {
    for path in watched.drain() {
        let _ = watcher.unwatch(&path);
    }
    watch_directories(watcher, directories, watched);
}

fn apply_event(
    event: notify::Event,
    roots: &[PathBuf],
    mut watcher: Option<&mut RecommendedWatcher>,
    watched: &mut std::collections::HashSet<PathBuf>,
) {
    for path in event.paths {
        if !is_within_roots(&path, roots) || !should_refresh_for_path(&path) {
            continue;
        }
        if path.exists() {
            if path.is_dir() {
                if let Ok(metadata) = fs::metadata(&path) {
                    upsert_indexed_file(path.clone(), mtime_secs_from_metadata(&metadata));
                }
                let Some(max_depth) = remaining_depth(&path, roots) else {
                    continue;
                };
                let (entries, directories) = scan_subtree(&path, max_depth);
                if let Ok(mut index) = FILE_INDEX.write() {
                    let index = Arc::make_mut(&mut index);
                    index.retain(|(existing, _)| !existing.starts_with(&path));
                    index.extend(entries);
                }
                if let Some(watcher) = watcher.as_deref_mut() {
                    watch_directories(watcher, directories, watched);
                }
            } else if let Ok(metadata) = fs::metadata(&path) {
                if metadata.is_file() {
                    upsert_indexed_file(path, mtime_secs_from_metadata(&metadata));
                }
            }
        } else {
            let stale: Vec<_> = watched
                .iter()
                .filter(|existing| existing.starts_with(&path))
                .cloned()
                .collect();
            if let Some(watcher) = watcher.as_deref_mut() {
                for stale_path in &stale {
                    let _ = watcher.unwatch(stale_path);
                }
            }
            for stale_path in stale {
                watched.remove(&stale_path);
            }
            if let Ok(mut index) = FILE_INDEX.write() {
                Arc::make_mut(&mut index).retain(|(existing, _)| !existing.starts_with(&path));
            }
        }
    }
}

fn build_index_snapshot(roots: &[PathBuf]) -> (Vec<IndexedFile>, Vec<PathBuf>) {
    let mut files = Vec::new();
    let mut directories = Vec::new();
    let mut seen_files = std::collections::HashSet::new();
    let mut seen_directories = std::collections::HashSet::new();

    for root in roots {
        let (root_files, root_directories) = scan_subtree(root, MAX_DEPTH);
        for (path, modified) in root_files {
            if seen_files.insert(path.clone()) {
                files.push((path, modified));
            }
        }
        for path in root_directories {
            if seen_directories.insert(path.clone()) {
                directories.push(path);
            }
        }
    }
    (files, directories)
}

fn scan_subtree(root: &Path, max_depth: usize) -> (Vec<IndexedFile>, Vec<PathBuf>) {
    let mut files = Vec::new();
    let mut directories = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_str().unwrap_or("");
            entry.depth() == 0 || (!name.starts_with('.') && !SKIP_DIRS.contains(&name))
        })
        .filter_map(Result::ok)
    {
        let path = entry.path().to_path_buf();
        if entry.file_type().is_dir() {
            files.push((path.clone(), mtime_secs(&path)));
            if entry.depth() < max_depth {
                directories.push(path);
            }
        } else if entry.file_type().is_file() {
            files.push((path.clone(), mtime_secs(&path)));
        }
    }
    (files, directories)
}

fn is_within_roots(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn remaining_depth(path: &Path, roots: &[PathBuf]) -> Option<usize> {
    let root = roots
        .iter()
        .filter(|root| path.starts_with(root.as_path()))
        .max_by_key(|root| root.components().count())?;
    let depth = path.strip_prefix(root).ok()?.components().count();
    (depth < MAX_DEPTH).then_some(MAX_DEPTH - depth)
}

fn mtime_secs_from_metadata(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn upsert_indexed_file(path: PathBuf, modified: u64) {
    if let Ok(mut index) = FILE_INDEX.write() {
        let files = Arc::make_mut(&mut index);
        if let Some(entry) = files.iter_mut().find(|(existing, _)| *existing == path) {
            entry.1 = modified;
        } else {
            files.push((path, modified));
        }
    }
}

fn replace_index(entries: Vec<IndexedFile>) {
    if let Ok(mut index) = FILE_INDEX.write() {
        *index = Arc::new(entries);
    }
}

fn should_refresh_for_event(event: &notify::Event) -> bool {
    let changes_index = matches!(
        &event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    );

    changes_index && event.paths.iter().any(|path| should_refresh_for_path(path))
}

fn should_refresh_for_path(path: &Path) -> bool {
    let is_user_scripts_path = USER_SCRIPTS_DIR
        .as_ref()
        .is_some_and(|scripts_dir| path.starts_with(scripts_dir));
    path.components().all(|component| {
        let Some(name) = component.as_os_str().to_str() else {
            return true;
        };
        (is_user_scripts_path || !name.starts_with('.')) && !SKIP_DIRS.contains(&name)
    })
}

/// Trigger an immediate index rebuild outside the normal refresh cycle.
pub fn rebuild_index_now() {
    std::thread::spawn(|| {
        let entries = build_index_entries();
        if let Ok(mut idx) = FILE_INDEX.write() {
            *idx = Arc::new(entries);
        }
    });
}

fn build_index_entries() -> Vec<(PathBuf, u64)> {
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for base in FileSearcher::search_paths() {
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
            if seen.insert(path.clone()) {
                entries.push((path.clone(), mtime_secs(&path)));
            }
        }
    }
    entries
}

pub struct FileSearcher;

impl FileSearcher {
    fn recent_files() -> Vec<ResultItem> {
        let Some(path) = dirs::data_local_dir().map(|dir| dir.join("recently-used.xbel")) else {
            return vec![];
        };
        let Ok(contents) = fs::read_to_string(path) else {
            return vec![];
        };

        let mut reader = Reader::from_str(&contents);
        reader.config_mut().trim_text(true);
        let mut recent = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(bookmark)) | Ok(Event::Empty(bookmark))
                    if bookmark.name().as_ref() == "bookmark" =>
                {
                    let mut href = None;
                    let mut visited = None;
                    let mut modified = None;
                    let mut added = None;
                    for attribute in bookmark.attributes().flatten() {
                        let value = match attribute.normalized_value(XmlVersion::Implicit1_0) {
                            Ok(value) => value.into_owned(),
                            Err(_) => continue,
                        };
                        match attribute.key.as_ref() {
                            "href" => href = Some(value),
                            "visited" => visited = Some(value),
                            "modified" => modified = Some(value),
                            "added" => added = Some(value),
                            _ => {}
                        }
                    }

                    if let Some(path) = href.as_deref().and_then(Self::path_from_file_url) {
                        let timestamp = visited
                            .as_deref()
                            .or(modified.as_deref())
                            .or(added.as_deref())
                            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                            .map(|value| value.timestamp())
                            .unwrap_or_default();
                        recent.push((timestamp, path));
                    }
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }

        recent.sort_by_key(|(visited, _)| std::cmp::Reverse(*visited));
        let mut seen = std::collections::HashSet::new();
        recent
            .into_iter()
            .filter_map(|(_, path)| {
                if !path.exists() || !seen.insert(path.clone()) {
                    return None;
                }
                Some(Self::path_to_result(path))
            })
            .take(MAX_RESULTS)
            .collect()
    }

    fn path_from_file_url(href: &str) -> Option<PathBuf> {
        let path = href.strip_prefix("file://")?;
        let path = path
            .strip_prefix("localhost/")
            .map(|path| format!("/{path}"))
            .unwrap_or_else(|| path.to_string());
        if !Path::new(&path).is_absolute() {
            return None;
        }
        let decoded = urlencoding::decode(&path).ok()?;
        Some(PathBuf::from(decoded.as_ref()))
    }

    fn search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let home = dirs::home_dir();

        // A home-directory scan already covers its standard folders. Watching
        // them separately duplicated scans and generated duplicate events.
        paths.extend(
            [
                dirs::download_dir(),
                dirs::document_dir(),
                dirs::desktop_dir(),
            ]
            .into_iter()
            .flatten()
            .filter(|path| match home.as_ref() {
                Some(home) => !path.starts_with(home),
                None => true,
            }),
        );
        if let Some(home) = home {
            paths.push(home);
        }

        // Include the user scripts directory for script discovery
        if let Some(scripts_dir) = USER_SCRIPTS_DIR.as_ref().cloned() {
            if !paths.contains(&scripts_dir) {
                paths.push(scripts_dir);
            }
        }

        paths.sort();
        paths.dedup();
        paths
    }

    fn resolve_explicit_path(query: &str) -> Option<PathBuf> {
        let expanded = if query.starts_with("~/") {
            crate::search_utils::expand_tilde(query)?
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
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        entries.into_iter().map(|(p, _)| p).collect()
    }

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
                (None, None) => return None,
                (a, b) => a.unwrap_or(0).max(b.unwrap_or(0)),
            };
            total += word_score;
        }
        Some(total)
    }

    fn collect_candidates(query_words: &[&str]) -> Vec<(PathBuf, i64)> {
        // Fast path: query the in-memory index
        let index = FILE_INDEX.read().ok().map(|entries| Arc::clone(&entries));
        if let Some(entries) = index.filter(|entries| !entries.is_empty()) {
            use rayon::prelude::*;
            let mut hits: Vec<(PathBuf, i64, u64)> = entries
                .par_iter()
                .filter_map(|(path, mtime)| {
                    Self::score(query_words, path).map(|s| (path.clone(), s, *mtime))
                })
                .collect();
            hits.sort_unstable_by_key(|(_, score, mtime)| {
                (std::cmp::Reverse(*score), std::cmp::Reverse(*mtime))
            });
            return hits.into_iter().map(|(p, s, _)| (p, s)).collect();
        }

        // Fallback: walk the filesystem directly (index not ready yet)
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
                if !seen.insert(path.clone()) {
                    continue;
                }
                if let Some(score) = Self::score(query_words, &path) {
                    hits.push((path, score));
                }
            }
        }

        sort_hits_by_score_and_mtime(hits)
    }

    fn icon_for(path: &Path) -> &'static str {
        if path.is_dir() {
            super::ICON_FOLDER
        } else {
            super::ICON_FILE
        }
    }

    fn path_to_result(path: PathBuf) -> ResultItem {
        let path_str = path.to_string_lossy().into_owned();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();
        let icon = Self::icon_for(&path);

        let parent_dir = path.parent().map(|p| p.to_string_lossy().into_owned());

        let open_action = Action::new(
            "Open",
            ActionData::OpenUrl {
                url: format!("file://{}", path_str),
            },
        );

        let mut actions: Vec<Action> = Vec::new();

        // Scripts: Run is the primary action, Open is secondary
        if is_script(&path) {
            actions.push(Action::new(
                "Run",
                ActionData::RunScript {
                    path: path_str.clone(),
                },
            ));
            actions.push(open_action);
        } else {
            actions.push(open_action);
        }

        // Folders: Open in Terminal
        if path.is_dir() {
            actions.push(Action::new(
                "Open in Terminal",
                ActionData::OpenInTerminal {
                    path: path_str.clone(),
                },
            ));
        }

        // Open containing folder
        if let Some(dir) = &parent_dir {
            actions.push(Action::new(
                "Open Containing Folder",
                ActionData::OpenUrl {
                    url: format!("file://{}", dir),
                },
            ));
        }

        // Move to Trash
        actions.push(Action::new(
            "Move to Trash",
            ActionData::RunFunction {
                function_name: "trash_file".into(),
                params: vec![path_str.clone()],
            },
        ));

        // Copy full path
        actions.push(Action::new(
            "Copy Path",
            ActionData::CopyToClipboard {
                text: path_str.clone(),
            },
        ));

        // Copy filename
        actions.push(Action::new(
            "Copy Filename",
            ActionData::CopyToClipboard { text: name.clone() },
        ));

        let mut item = ResultItem::new(name, actions)
            .description(path_str.clone())
            .icon(icon);
        if !path.is_dir() {
            item = item.draggable_path(path_str);
        }
        item
    }
}

/// Gets the user scripts directory from config, creating it if necessary.
fn get_user_scripts_dir() -> Option<PathBuf> {
    let dir = super::scripts::scripts_dir().or_else(|| {
        // scripts_dir() returns None if the path doesn't exist yet; create it first.
        let configured = crate::CONFIG.read().ok()?.scripts.path.clone();
        let path = crate::search_utils::expand_tilde(&configured)?;
        fs::create_dir_all(&path).ok()?;
        Some(path)
    })?;
    Some(dir)
}

/// Returns true if the file should be treated as a runnable script.
/// Only scripts in the user's ~/.config/quarry/scripts directory are allowed.
fn is_script(path: &Path) -> bool {
    if path.is_dir() {
        return false;
    }

    // Only allow scripts from the user scripts directory
    let Some(scripts_dir) = USER_SCRIPTS_DIR.as_ref() else {
        return false;
    };

    // Check if the file is in the scripts directory
    if !path.starts_with(&scripts_dir) {
        return false;
    }

    // Check extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if SCRIPT_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
            return true;
        }
    }

    // On Unix, also treat executable files with no extension as scripts
    // (e.g. a compiled-away shebang script named "deploy")
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode();
            // owner, group, or other execute bit set
            if mode & 0o111 != 0 {
                return true;
            }
        }
    }

    false
}

impl SearchProvider for FileSearcher {
    fn name(&self) -> String {
        "files".to_string()
    }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return SearchResult::list(Self::recent_files());
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
                return SearchResult::list(results);
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
                    return SearchResult::list(results);
                } else {
                    return SearchResult::list(vec![Self::path_to_result(explicit)]);
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
            return SearchResult::list(vec![]);
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
                if !seen.insert(path.clone()) {
                    continue;
                }
                if let Some(score) = Self::score(&query_words, &path) {
                    hits.push((path, score));
                }
            }
            sort_hits_by_score_and_mtime(hits)
        } else {
            Self::collect_candidates(&query_words)
        };

        let results: Vec<ResultItem> = candidates
            .into_iter()
            .take(MAX_RESULTS)
            .map(|(path, _)| Self::path_to_result(path))
            .collect();

        SearchResult::list(results)
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

fn sort_hits_by_score_and_mtime(hits: Vec<(PathBuf, i64)>) -> Vec<(PathBuf, i64)> {
    let mut hits: Vec<_> = hits
        .into_iter()
        .map(|(path, score)| {
            let modified = mtime_secs(&path);
            (path, score, modified)
        })
        .collect();
    hits.sort_unstable_by_key(|(_, score, modified)| {
        (std::cmp::Reverse(*score), std::cmp::Reverse(*modified))
    });
    hits.into_iter()
        .map(|(path, score, _)| (path, score))
        .collect()
}
