use base64::{engine::general_purpose, Engine};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

const MAX_RESULTS: usize = 20;
const THUMBNAIL_MAX: u32 = 220;

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

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

    // Cache thumbnails to disk keyed by path+mtime so they survive restarts.
    // First open is slow; every open after is a cheap file read.
    fn thumbnail_cache_path(img_path: &Path, mtime: u64) -> PathBuf {
        let key = path_hash(img_path, mtime);
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("quarry/screenshots")
            .join(format!("{:016x}.jpg", key))
    }

    fn make_thumbnail(path: &Path, mtime: u64) -> Option<String> {
        let cache = Self::thumbnail_cache_path(path, mtime);

        // Fast path: serve from disk cache
        if cache.exists() {
            if let Ok(bytes) = std::fs::read(&cache) {
                return Some(format!(
                    "data:image/jpeg;base64,{}",
                    general_purpose::STANDARD.encode(&bytes)
                ));
            }
        }

        // Slow path: decode full image, resize, save JPEG to cache
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

        let size_str = Self::file_size(&path)
            .map(Self::format_size)
            .unwrap_or_default();
        let dims_str = Self::image_dimensions(&path)
            .map(|(w, h)| format!("{}×{}", w, h))
            .unwrap_or_default();
        let meta = format!("{}|{}|{}", Self::format_mtime(mtime), dims_str, size_str);

        let item = ResultItem::new(
            name,
            vec![
                Action::new(
                    "Open",
                    ActionData::OpenUrl {
                        url: format!("file://{}", path_str),
                    },
                ),
                Action::new(
                    "Copy Screenshot",
                    ActionData::CopyImageFile { path: path_str.clone() },
                ),
                Action::new(
                    "Copy Path",
                    ActionData::CopyToClipboard {
                        text: path_str.clone(),
                    },
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

        let results: Vec<ResultItem> = if q.is_empty() {
            files
                .into_iter()
                .take(MAX_RESULTS)
                .map(|(p, m)| Self::to_result(p, m))
                .collect()
        } else {
            files
                .into_iter()
                .filter_map(|(path, mtime)| {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    MATCHER.fuzzy_match(&name, q).map(|_| (path, mtime))
                })
                .take(MAX_RESULTS)
                .map(|(p, m)| Self::to_result(p, m))
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
