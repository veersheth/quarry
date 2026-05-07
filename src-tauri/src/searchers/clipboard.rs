use once_cell::sync::Lazy;
use std::sync::Mutex;

use super::SearchProvider;
use crate::clipboard_manager::ClipboardContent;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use crate::CLIPBOARD_MANAGER;
use tauri::AppHandle;

// Cache for the empty-query result, keyed by (history generation, pin count).
// Pin count changes whenever pins are added or removed, invalidating the cache.
static CACHE: Lazy<Mutex<((u64, usize), Option<SearchResult>)>> =
    Lazy::new(|| Mutex::new(((u64::MAX, 0), None)));

fn cache_key() -> (u64, usize) {
    let gen = CLIPBOARD_MANAGER.generation();
    let pins = crate::PINS.get("clipboard").len() + crate::PINS.get("clipboard_image").len();
    (gen, pins)
}

pub struct ClipboardSearcher;

pub fn warm() {
    let result = build_results("");
    let key = cache_key();
    if let Ok(mut cache) = CACHE.lock() {
        *cache = (key, Some(result));
    }
}

impl SearchProvider for ClipboardSearcher {
    fn name(&self) -> String { "clipboard".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim().to_lowercase();

        if query == "clear" {
            return SearchResult {
                results: vec![ResultItem::new(
                    "Clear clipboard history?",
                    vec![Action::new("Clear", ActionData::RunFunction {
                        function_name: "clear_clipboard".into(),
                        params: vec![],
                    })],
                )],
                result_type: ResultType::List,
                            ..Default::default()
};
        }

        // Serve from cache for empty queries when neither history nor pins changed.
        if query.is_empty() {
            let key = cache_key();
            if let Ok(cache) = CACHE.lock() {
                if cache.0 == key {
                    if let Some(ref cached) = cache.1 {
                        return cached.clone();
                    }
                }
            }
        }

        let result = build_results(&query);

        if query.is_empty() {
            let key = cache_key();
            if let Ok(mut cache) = CACHE.lock() {
                *cache = (key, Some(result.clone()));
            }
        }

        result
    }
}

fn build_results(query: &str) -> SearchResult {
    let text_pins = crate::PINS.get("clipboard");
    let pinned_texts: std::collections::HashSet<&str> =
        text_pins.iter().map(|p| p.payload.as_str()).collect();

    let image_pins = crate::PINS.get("clipboard_image");
    let pinned_hashes: std::collections::HashSet<u64> = image_pins
        .iter()
        .filter_map(|p| p.name.parse::<u64>().ok())
        .collect();

    let mut results: Vec<ResultItem> = Vec::new();

    // Prepend pinned entries on empty query (survive history eviction)
    if query.is_empty() {
        for pin in &text_pins {
            results.push(
                ResultItem::new(
                    pin.payload.clone(),
                    vec![
                        Action::new("Copy", ActionData::CopyToClipboard { text: pin.payload.clone() }),
                        Action::new("Unpin", ActionData::RunFunction {
                            function_name: "unpin".into(),
                            params: vec!["clipboard".into(), pin.name.clone()],
                        }),
                    ],
                )
                .description("pinned".to_string())
                .pinned(),
            );
        }

        for pin in &image_pins {
            if let Ok(img) = serde_json::from_str::<serde_json::Value>(&pin.payload) {
                let full = img["full"].as_str().unwrap_or("").to_string();
                let thumbnail = img["thumbnail"].as_str().unwrap_or("").to_string();
                let width = img["width"].as_u64().unwrap_or(0) as u32;
                let height = img["height"].as_u64().unwrap_or(0) as u32;
                results.push(
                    ResultItem::new(
                        format!("Image {}×{}", width, height),
                        vec![
                            Action::new("Copy", ActionData::CopyImageToClipboard {
                                base64_png: full,
                                width,
                                height,
                            }),
                            Action::new("Unpin", ActionData::RunFunction {
                                function_name: "unpin".into(),
                                params: vec!["clipboard_image".into(), pin.name.clone()],
                            }),
                        ],
                    )
                    .description("pinned".to_string())
                    .thumbnail(thumbnail)
                    .pinned(),
                );
            }
        }
    }

    // Build history items while holding the lock — no full-image clone needed.
    // The `full` field is intentionally not accessed; copy actions reference by hash.
    CLIPBOARD_MANAGER.with_history(|history| {
        let history_items: Vec<ResultItem> = history
            .iter()
            .filter(|entry| {
                if query.is_empty() {
                    return match &entry.content {
                        ClipboardContent::Text { value } => !pinned_texts.contains(value.as_str()),
                        ClipboardContent::Image { hash, .. } => !pinned_hashes.contains(hash),
                    };
                }
                entry.display_text().to_lowercase().contains(query)
            })
            .map(|entry| match &entry.content {
                ClipboardContent::Text { value } => {
                    let is_pinned = crate::PINS.contains("clipboard", value);
                    let pin_action = if is_pinned {
                        Action::new("Unpin", ActionData::RunFunction {
                            function_name: "unpin".into(),
                            params: vec!["clipboard".into(), value.clone()],
                        })
                    } else {
                        Action::new("Pin", ActionData::RunFunction {
                            function_name: "pin".into(),
                            params: vec!["clipboard".into(), value.clone(), value.clone()],
                        })
                    };
                    let (icon, path_actions) = detect_path(value);
                    let mut actions = vec![
                        Action::new("Copy", ActionData::CopyToClipboard { text: value.clone() }),
                    ];
                    actions.extend(path_actions);
                    actions.extend(transform_actions(value));
                    actions.extend([
                        pin_action,
                        Action::new("Delete", ActionData::RunFunction {
                            function_name: "delete_clipboard_entry".into(),
                            params: vec![entry.timestamp.to_string()],
                        }),
                        Action::new("Clear entire clipboard", ActionData::RunFunction {
                            function_name: "clear_clipboard".into(),
                            params: vec![],
                        }),
                    ]);
                    let mut item = ResultItem::new(value.clone(), actions)
                        .description(format_timestamp(entry.timestamp));
                    if let Some(icon) = icon {
                        item = item.icon(icon);
                    }
                    item
                }

                ClipboardContent::Image { thumbnail, width, height, ocr_text, hash, .. } => {
                    let hash_str = hash.to_string();
                    let is_pinned = crate::PINS.contains("clipboard_image", &hash_str);
                    let pin_action = if is_pinned {
                        Action::new("Unpin", ActionData::RunFunction {
                            function_name: "unpin".into(),
                            params: vec!["clipboard_image".into(), hash_str.clone()],
                        })
                    } else {
                        // Full image is fetched lazily at pin time, not here.
                        Action::new("Pin", ActionData::RunFunction {
                            function_name: "pin_clipboard_image".into(),
                            params: vec![hash_str.clone()],
                        })
                    };

                    let mut actions = vec![
                        Action::new("Copy", ActionData::RunFunction {
                            function_name: "copy_clipboard_image".into(),
                            params: vec![hash_str],
                        }),
                    ];
                    if let Some(text) = ocr_text {
                        actions.push(Action::new("Copy Text", ActionData::CopyToClipboard { text: text.clone() }));
                    }
                    actions.extend([
                        pin_action,
                        Action::new("Delete", ActionData::RunFunction {
                            function_name: "delete_clipboard_entry".into(),
                            params: vec![entry.timestamp.to_string()],
                        }),
                        Action::new("Clear entire clipboard", ActionData::RunFunction {
                            function_name: "clear_clipboard".into(),
                            params: vec![],
                        }),
                    ]);

                    let mut item = ResultItem::new(format!("Image {}×{}", width, height), actions)
                        .description(format_timestamp(entry.timestamp))
                        .thumbnail(thumbnail.clone());
                    if let Some(text) = ocr_text {
                        item = item.ocr_text(text.clone());
                    }
                    item
                }
            })
            .collect();
        results.extend(history_items);
    });

    SearchResult {
        results,
        result_type: ResultType::Clipboard,
            ..Default::default()
}
}

fn resolve_path(value: &str) -> Option<std::path::PathBuf> {
    let trimmed = value.trim();
    if !trimmed.starts_with('/') && !trimmed.starts_with("~/") {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        Some(dirs::home_dir()?.join(rest))
    } else {
        Some(std::path::PathBuf::from(trimmed))
    }
}

fn detect_path(value: &str) -> (Option<&'static str>, Vec<Action>) {
    let Some(path) = resolve_path(value) else {
        return (None, vec![]);
    };
    let path_str = path.to_string_lossy().into_owned();

    if path.is_dir() {
        let actions = vec![
            Action::new("Open Folder", ActionData::OpenUrl { url: format!("file://{}", path_str) }),
        ];
        (Some("icons/folder.png"), actions)
    } else if path.is_file() {
        let parent = path.parent()
            .map(|p| format!("file://{}", p.to_string_lossy()));
        let mut actions = vec![
            Action::new("Open", ActionData::OpenUrl { url: format!("file://{}", path_str) }),
        ];
        if let Some(parent_url) = parent {
            actions.push(Action::new("Open Containing Folder", ActionData::OpenUrl { url: parent_url }));
        }
        (Some("icons/file.png"), actions)
    } else {
        (None, vec![])
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn transform_actions(value: &str) -> Vec<Action> {
    use base64::{engine::general_purpose, Engine};

    let mut actions: Vec<Action> = Vec::new();

    let upper = value.to_uppercase();
    if upper != value {
        actions.push(Action::new("Uppercase", ActionData::CopyToClipboard { text: upper }));
    }

    let lower = value.to_lowercase();
    if lower != value {
        actions.push(Action::new("Lowercase", ActionData::CopyToClipboard { text: lower }));
    }

    let trimmed = value.trim().to_string();
    if trimmed != value {
        actions.push(Action::new("Trim whitespace", ActionData::CopyToClipboard { text: trimmed }));
    }

    let titled = title_case(value);
    if titled != value {
        actions.push(Action::new("Title Case", ActionData::CopyToClipboard { text: titled }));
    }

    let encoded = urlencoding::encode(value).to_string();
    if encoded != value {
        actions.push(Action::new("URL encode", ActionData::CopyToClipboard { text: encoded }));
    }

    if let Ok(decoded) = urlencoding::decode(value) {
        let decoded = decoded.into_owned();
        if decoded != value {
            actions.push(Action::new("URL decode", ActionData::CopyToClipboard { text: decoded }));
        }
    }

    let b64 = general_purpose::STANDARD.encode(value.as_bytes());
    actions.push(Action::new("Base64 encode", ActionData::CopyToClipboard { text: b64 }));

    if let Ok(decoded) = general_purpose::STANDARD.decode(value.trim().as_bytes()) {
        if let Ok(s) = String::from_utf8(decoded) {
            actions.push(Action::new("Base64 decode", ActionData::CopyToClipboard { text: s }));
        }
    }

    actions
}

fn format_timestamp(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let age = now.saturating_sub(ts);
    if age < 60 {
        "just now".to_string()
    } else if age < 3600 {
        format!("{} min ago", age / 60)
    } else if age < 86400 {
        format!("{} hr ago", age / 3600)
    } else {
        format!("{} days ago", age / 86400)
    }
}
