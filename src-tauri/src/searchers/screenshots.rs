use base64::{engine::general_purpose, Engine};
use dashmap::DashSet;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

/// Maximum screenshots rendered with thumbnails. Text search scans all OCR
/// caches (cheap disk reads) but only builds thumbnails for the top matches.
const MAX_DISPLAY: usize = 30;
/// Maximum concurrent tesseract jobs. New files are picked up on later searches.
const MAX_OCR_JOBS: usize = 4;
const THUMBNAIL_MAX: u32 = 220;

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

// Tracks paths currently being OCR'd so we don't spawn duplicate jobs.
static OCR_IN_PROGRESS: Lazy<DashSet<PathBuf>> = Lazy::new(DashSet::new);

pub struct ScreenshotsSearcher;

impl ScreenshotsSearcher {
    fn dir() -> PathBuf {
        let path = crate::CONFIG.read().unwrap().screenshots.path.clone();
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest);
            }
        }
        let p = PathBuf::from(&path);
        if p.is_absolute() {
            return p;
        }
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Pictures/Screenshots")
    }

    fn is_image(path: &Path) -> bool {
        matches!(
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .as_deref(),
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif")
        )
    }

    fn mtime(path: &Path) -> u64 {
        std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn format_mtime(secs: u64) -> String {
        use chrono::TimeZone;
        chrono::Local
            .timestamp_opt(secs as i64, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default()
    }

    fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("quarry/screenshots")
    }

    fn thumbnail_cache_path(img_path: &Path, mtime: u64) -> PathBuf {
        Self::cache_dir().join(format!("{:016x}.jpg", path_hash(img_path, mtime)))
    }

    fn ocr_cache_path(img_path: &Path, mtime: u64) -> PathBuf {
        Self::cache_dir().join(format!("{:016x}.txt", path_hash(img_path, mtime)))
    }

    fn make_thumbnail(path: &Path, mtime: u64) -> Option<String> {
        let cache = Self::thumbnail_cache_path(path, mtime);

        if cache.exists() {
            if let Ok(bytes) = std::fs::read(&cache) {
                return Some(format!(
                    "data:image/jpeg;base64,{}",
                    general_purpose::STANDARD.encode(&bytes)
                ));
            }
        }

        let img = image::open(path).ok()?;
        let thumb = img.thumbnail(THUMBNAIL_MAX, THUMBNAIL_MAX * 3);
        let mut buf = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut buf);
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 82);
            enc.encode_image(&thumb).ok()?;
        }
        if let Some(parent) = cache.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&cache, &buf).ok();

        Some(format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(&buf)
        ))
    }

    fn read_ocr_cache(path: &Path, mtime: u64) -> Option<String> {
        let cache = Self::ocr_cache_path(path, mtime);
        std::fs::read_to_string(&cache).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// Spawn a background thread to OCR `path`. Skips if already cached,
    /// already in progress, or the concurrent job limit is reached (the file
    /// will be picked up on the next search).
    fn queue_ocr(path: PathBuf, mtime: u64) {
        let cache = Self::ocr_cache_path(&path, mtime);
        if cache.exists() || OCR_IN_PROGRESS.contains(&path) {
            return;
        }
        if OCR_IN_PROGRESS.len() >= MAX_OCR_JOBS {
            return;
        }
        OCR_IN_PROGRESS.insert(path.clone());
        std::thread::spawn(move || {
            let result = std::process::Command::new("tesseract")
                .args([path.to_str().unwrap_or(""), "stdout", "--psm", "3", "-l", "eng"])
                .output();
            if let Ok(output) = result {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() {
                    if let Some(parent) = cache.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::write(&cache, &text).ok();
                }
            }
            OCR_IN_PROGRESS.remove(&path);
        });
    }

    fn list_files(dir: &Path) -> Vec<(PathBuf, u64)> {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return vec![];
        };
        let mut files: Vec<(PathBuf, u64)> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && Self::is_image(p))
            .map(|p| {
                let m = Self::mtime(&p);
                (p, m)
            })
            .collect();
        files.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        files
    }

    fn file_size(path: &Path) -> Option<u64> {
        std::fs::metadata(path).ok().map(|m| m.len())
    }

    fn format_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    fn image_dimensions(path: &Path) -> Option<(u32, u32)> {
        image::image_dimensions(path).ok()
    }

    fn to_result(path: PathBuf, mtime: u64) -> ResultItem {
        let path_str = path.to_string_lossy().into_owned();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();

        let size_str = Self::file_size(&path).map(Self::format_size).unwrap_or_default();
        let dims_str = Self::image_dimensions(&path)
            .map(|(w, h)| format!("{}×{}", w, h))
            .unwrap_or_default();
        let meta = format!("{}|{}|{}", Self::format_mtime(mtime), dims_str, size_str);

        let item = ResultItem::new(
            name,
            vec![
                Action::new(
                    "Open",
                    ActionData::OpenUrl { url: format!("file://{}", path_str) },
                ),
                Action::new(
                    "Copy Screenshot",
                    ActionData::CopyImageFile { path: path_str.clone() },
                ),
                Action::new(
                    "Copy Path",
                    ActionData::CopyToClipboard { text: path_str.clone() },
                ),
                Action::new(
                    "Extract Text",
                    ActionData::RunFunction {
                        function_name: "ocr_screenshot".into(),
                        params: vec![path_str.clone()],
                    },
                ),
                Action::new(
                    "Delete",
                    ActionData::RunFunction {
                        function_name: "delete_file".into(),
                        params: vec![path_str],
                    },
                ),
            ],
        )
        .description(meta);

        match Self::make_thumbnail(&path, mtime) {
            Some(thumb) => item.thumbnail(thumb),
            None => item,
        }
    }
}

impl SearchProvider for ScreenshotsSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let dir = Self::dir();
        let q = query.trim();
        let files = Self::list_files(&dir);

        // Queue background OCR for any file that doesn't have cached text yet.
        for (path, mtime) in &files {
            Self::queue_ocr(path.clone(), *mtime);
        }

        let results: Vec<ResultItem> = if q.is_empty() {
            // No query: show most recent, thumbnail only for displayed items.
            files
                .into_iter()
                .take(MAX_DISPLAY)
                .map(|(p, m)| Self::to_result(p, m))
                .collect()
        } else {
            // Query: read OCR caches for ALL files (cheap disk reads), score
            // and rank, then build thumbnails only for the top matches.
            let mut scored: Vec<(PathBuf, u64, i64)> = files
                .into_iter()
                .filter_map(|(path, mtime)| {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let ocr = Self::read_ocr_cache(&path, mtime).unwrap_or_default();

                    let name_score = MATCHER.fuzzy_match(&name, q).map(|s| s * 3);
                    let ocr_score = if ocr.is_empty() {
                        None
                    } else {
                        MATCHER.fuzzy_match(&ocr, q)
                    };

                    match (name_score, ocr_score) {
                        (None, None) => None,
                        (a, b) => Some((path, mtime, a.unwrap_or(0).max(b.unwrap_or(0)))),
                    }
                })
                .collect();

            scored.sort_unstable_by(|a, b| b.2.cmp(&a.2));
            scored
                .into_iter()
                .take(MAX_DISPLAY)
                .map(|(p, m, _)| Self::to_result(p, m))
                .collect()
        };

        SearchResult {
            results,
            result_type: ResultType::Screenshots,
        }
    }
}

fn path_hash(path: &Path, mtime: u64) -> u64 {
    let s = format!("{}:{}", path.to_string_lossy(), mtime);
    s.bytes()
        .fold(14695981039346656037u64, |h, b| {
            h.wrapping_mul(1099511628211) ^ b as u64
        })
}
