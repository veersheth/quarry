use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use super::SearchProvider;
use crate::clipboard_manager::ClipboardContent;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use crate::CLIPBOARD_MANAGER;
use tauri::AppHandle;

// Cache path-detection results so repeated stat() calls are avoided on re-warm.
// Key: text value. Value: (icon, has_open_action, has_folder_action).
static PATH_CACHE: Lazy<Mutex<HashMap<String, Option<&'static str>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Cache for the empty-query result, keyed by (history generation, pin count).
// Pin count changes whenever pins are added or removed, invalidating the cache.
type ClipboardCache = ((u64, usize), Option<SearchResult>);

static CACHE: Lazy<Mutex<ClipboardCache>> = Lazy::new(|| Mutex::new(((u64::MAX, 0), None)));

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
    fn name(&self) -> String {
        "clipboard".to_string()
    }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        if !CLIPBOARD_MANAGER.is_monitoring() {
            return SearchResult::list(vec![ResultItem::new(
                "Start clipboard tracking",
                vec![Action::new(
                    "Start tracking",
                    ActionData::RunFunction {
                        function_name: "enable_clipboard_tracking".into(),
                        params: vec![],
                    },
                )],
            )
            .description("Save future clipboard items in local history")]);
        }
        let query = query.trim().to_lowercase();

        if query == "clear" {
            return SearchResult::list(vec![ResultItem::new(
                "Clear clipboard history?",
                vec![Action::new(
                    "Clear",
                    ActionData::RunFunction {
                        function_name: "clear_clipboard".into(),
                        params: vec![],
                    },
                )],
            )]);
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

/// Max chars of text included in the IPC payload for display.
/// Full text is stored in ACTION_REGISTRY and used when copying or shown lazily in the preview.
const DISPLAY_CHARS: usize = 500;

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

    if query.is_empty() {
        results.push(
            ResultItem::new(
                "Stop clipboard tracking",
                vec![Action::new(
                    "Stop tracking",
                    ActionData::RunFunction {
                        function_name: "disable_clipboard_tracking".into(),
                        params: vec![],
                    },
                )],
            )
            .description("Stop recording new clipboard items"),
        );
    }

    // Prepend pinned entries on empty query (survive history eviction)
    if query.is_empty() {
        for pin in &text_pins {
            let (icon, path_actions) = detect_path(&pin.payload);
            let mut actions = vec![
                Action::new(
                    "Paste",
                    ActionData::PasteToApp {
                        text: pin.payload.clone(),
                    },
                ),
                Action::new(
                    "Copy",
                    ActionData::CopyToClipboard {
                        text: pin.payload.clone(),
                    },
                ),
            ];
            actions.extend(path_actions);
            actions.extend(path_copy_actions(&pin.payload, icon));
            actions.extend(smart_actions(&pin.payload));
            actions.extend(transform_actions(&pin.payload));
            actions.push(Action::new(
                "Unpin",
                ActionData::RunFunction {
                    function_name: "unpin".into(),
                    params: vec!["clipboard".into(), pin.name.clone()],
                },
            ));
            let mut item = ResultItem::new(pin.payload.clone(), actions)
                .description("pinned".to_string())
                .pinned();
            if let Some(icon) = icon {
                item = item.icon(icon);
            }
            results.push(item);
        }

        for pin in &image_pins {
            if let Ok(img) = serde_json::from_str::<serde_json::Value>(&pin.payload) {
                let thumbnail = img["thumbnail"].as_str().unwrap_or("").to_string();
                let width = img["width"].as_u64().unwrap_or(0) as u32;
                let height = img["height"].as_u64().unwrap_or(0) as u32;
                results.push(
                    ResultItem::new(
                        format!("Image {}×{}", width, height),
                        vec![
                            Action::new(
                                "Paste",
                                ActionData::RunFunction {
                                    function_name: "paste_clipboard_image".into(),
                                    params: vec![pin.name.clone()],
                                },
                            ),
                            Action::new(
                                "Copy",
                                ActionData::RunFunction {
                                    function_name: "copy_clipboard_image".into(),
                                    params: vec![pin.name.clone()],
                                },
                            ),
                            Action::new(
                                "Unpin",
                                ActionData::RunFunction {
                                    function_name: "unpin".into(),
                                    params: vec!["clipboard_image".into(), pin.name.clone()],
                                },
                            ),
                        ],
                    )
                    .description("pinned".to_string())
                    .thumbnail(thumbnail)
                    .pinned(),
                );
            }
        }
    }

    // Build history items while holding the lock - no full-image clone needed.
    // The `full` field is intentionally not accessed; copy actions reference by hash.
    CLIPBOARD_MANAGER.with_history(|history| {
        // Deduplicate by content key (history is newest-first, so first seen = most recent).
        let mut seen_texts = std::collections::HashSet::new();
        let mut seen_hashes = std::collections::HashSet::new();

        // For a non-empty query: score each entry and sort by score descending.
        // For empty query: preserve history order (most recent first), no scoring.
        let mut scored: Vec<(i64, &_)> = history
            .iter()
            .filter_map(|entry| {
                // Deduplicate — skip if already seen a more recent copy of this content
                let is_new = match &entry.content {
                    ClipboardContent::Text { value } => seen_texts.insert(value.as_str()),
                    ClipboardContent::Image { hash, .. } => seen_hashes.insert(*hash),
                };
                if !is_new {
                    return None;
                }

                // Filter out pinned entries (shown above the history)
                let is_pinned = match &entry.content {
                    ClipboardContent::Text { value } => pinned_texts.contains(value.as_str()),
                    ClipboardContent::Image { hash, .. } => pinned_hashes.contains(hash),
                };
                if is_pinned {
                    return None;
                }

                if query.is_empty() {
                    Some((0i64, entry))
                } else {
                    // Score against the display text (and OCR text for images)
                    let text_score = crate::search_utils::smart_match(&entry.display_text(), query);
                    let ocr_score = match &entry.content {
                        ClipboardContent::Image {
                            ocr_text: Some(ocr),
                            ..
                        } => crate::search_utils::smart_match(ocr, query),
                        _ => None,
                    };
                    let score = match (text_score, ocr_score) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (Some(a), None) => Some(a),
                        (None, Some(b)) => Some(b),
                        (None, None) => None,
                    };
                    score.map(|s| (s, entry))
                }
            })
            .collect();

        if !query.is_empty() {
            scored.sort_unstable_by_key(|item| std::cmp::Reverse(item.0));
        }

        let history_items: Vec<ResultItem> = scored
            .into_iter()
            .map(|(_, entry)| match &entry.content {
                ClipboardContent::Text { value } => {
                    let is_pinned = crate::PINS.contains("clipboard", value);
                    let pin_action = if is_pinned {
                        Action::new(
                            "Unpin",
                            ActionData::RunFunction {
                                function_name: "unpin".into(),
                                params: vec!["clipboard".into(), value.clone()],
                            },
                        )
                    } else {
                        Action::new(
                            "Pin",
                            ActionData::RunFunction {
                                function_name: "pin".into(),
                                params: vec!["clipboard".into(), value.clone(), value.clone()],
                            },
                        )
                    };
                    let (icon, path_actions) = detect_path(value);
                    let mut actions = vec![
                        Action::new(
                            "Paste",
                            ActionData::PasteToApp {
                                text: value.clone(),
                            },
                        ),
                        Action::new(
                            "Copy",
                            ActionData::CopyToClipboard {
                                text: value.clone(),
                            },
                        ),
                    ];
                    actions.extend(path_actions);
                    actions.extend(path_copy_actions(value, icon));
                    actions.extend(smart_actions(value));
                    actions.extend(transform_actions(value));
                    actions.extend([
                        pin_action,
                        Action::new(
                            "Delete",
                            ActionData::RunFunction {
                                function_name: "delete_clipboard_entry".into(),
                                params: vec![entry.timestamp.to_string()],
                            },
                        ),
                        Action::new(
                            "Clear entire clipboard",
                            ActionData::RunFunction {
                                function_name: "clear_clipboard".into(),
                                params: vec![],
                            },
                        ),
                    ]);
                    // Truncate display name — full text lives in ACTION_REGISTRY for copying
                    // and is fetched lazily for the preview panel via get_action_text.
                    let display = if value.len() > DISPLAY_CHARS {
                        let mut s = value[..value.floor_char_boundary(DISPLAY_CHARS)].to_string();
                        s.push('…');
                        s
                    } else {
                        value.clone()
                    };
                    let mut item = ResultItem::new(display, actions)
                        .description(format_timestamp(entry.timestamp));
                    if let Some(icon) = icon {
                        item = item.icon(icon);
                    }
                    item
                }

                ClipboardContent::Image {
                    width,
                    height,
                    ocr_text,
                    hash,
                    ..
                } => {
                    let hash_str = hash.to_string();
                    let is_pinned = crate::PINS.contains("clipboard_image", &hash_str);
                    let pin_action = if is_pinned {
                        Action::new(
                            "Unpin",
                            ActionData::RunFunction {
                                function_name: "unpin".into(),
                                params: vec!["clipboard_image".into(), hash_str.clone()],
                            },
                        )
                    } else {
                        // Full image is fetched lazily at pin time, not here.
                        Action::new(
                            "Pin",
                            ActionData::RunFunction {
                                function_name: "pin_clipboard_image".into(),
                                params: vec![hash_str.clone()],
                            },
                        )
                    };

                    let mut actions = vec![
                        Action::new(
                            "Paste",
                            ActionData::RunFunction {
                                function_name: "paste_clipboard_image".into(),
                                params: vec![hash_str.clone()],
                            },
                        ),
                        Action::new(
                            "Copy",
                            ActionData::RunFunction {
                                function_name: "copy_clipboard_image".into(),
                                params: vec![hash_str.clone()],
                            },
                        ),
                    ];
                    if let Some(text) = ocr_text {
                        actions.push(Action::new(
                            "Copy Text",
                            ActionData::CopyToClipboard { text: text.clone() },
                        ));
                    }
                    actions.extend([
                        pin_action,
                        Action::new(
                            "Delete",
                            ActionData::RunFunction {
                                function_name: "delete_clipboard_entry".into(),
                                params: vec![entry.timestamp.to_string()],
                            },
                        ),
                        Action::new(
                            "Clear entire clipboard",
                            ActionData::RunFunction {
                                function_name: "clear_clipboard".into(),
                                params: vec![],
                            },
                        ),
                    ]);

                    // Thumbnail is fetched lazily by the frontend using thumbnail_key.
                    let mut item = ResultItem::new(format!("Image {}×{}", width, height), actions)
                        .description(format_timestamp(entry.timestamp))
                        .thumbnail_key(hash.to_string());
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
    crate::search_utils::expand_tilde(trimmed)
}

fn detect_path(value: &str) -> (Option<&'static str>, Vec<Action>) {
    let Some(path) = resolve_path(value) else {
        return (None, vec![]);
    };

    // Check path cache first to avoid repeated stat() syscalls on re-warm.
    if let Ok(cache) = PATH_CACHE.lock() {
        if let Some(icon) = cache.get(value) {
            // Rebuild actions from the cached icon type — no stat needed.
            let path_str = path.to_string_lossy().into_owned();
            return match icon {
                Some(i) if *i == super::ICON_FOLDER => (
                    *icon,
                    vec![Action::new(
                        "Open Folder",
                        ActionData::OpenUrl {
                            url: format!("file://{}", path_str),
                        },
                    )],
                ),
                Some(i) if *i == super::ICON_FILE => {
                    let parent = path
                        .parent()
                        .map(|p| format!("file://{}", p.to_string_lossy()));
                    let mut actions = vec![Action::new(
                        "Open",
                        ActionData::OpenUrl {
                            url: format!("file://{}", path_str),
                        },
                    )];
                    if let Some(parent_url) = parent {
                        actions.push(Action::new(
                            "Open Containing Folder",
                            ActionData::OpenUrl { url: parent_url },
                        ));
                    }
                    (*icon, actions)
                }
                _ => (None, vec![]),
            };
        }
    }

    // Cache miss — do the stat calls and store the result.
    let path_str = path.to_string_lossy().into_owned();
    let (icon, actions) = if path.is_dir() {
        (
            Some(super::ICON_FOLDER),
            vec![Action::new(
                "Open Folder",
                ActionData::OpenUrl {
                    url: format!("file://{}", path_str),
                },
            )],
        )
    } else if path.is_file() {
        let parent = path
            .parent()
            .map(|p| format!("file://{}", p.to_string_lossy()));
        let mut actions = vec![Action::new(
            "Open",
            ActionData::OpenUrl {
                url: format!("file://{}", path_str),
            },
        )];
        if let Some(parent_url) = parent {
            actions.push(Action::new(
                "Open Containing Folder",
                ActionData::OpenUrl { url: parent_url },
            ));
        }
        (Some(super::ICON_FILE), actions)
    } else {
        (None, vec![])
    };

    if let Ok(mut cache) = PATH_CACHE.lock() {
        cache.insert(value.to_string(), icon);
    }

    (icon, actions)
}

fn path_copy_actions(value: &str, icon: Option<&str>) -> Vec<Action> {
    if icon.is_none() {
        return vec![];
    }
    let Some(path) = resolve_path(value) else {
        return vec![];
    };
    let mut actions = Vec::new();
    if let Some(name) = path.file_name() {
        actions.push(Action::new(
            "Copy name",
            ActionData::CopyToClipboard {
                text: name.to_string_lossy().into_owned(),
            },
        ));
    }
    if let Some(parent) = path.parent() {
        actions.push(Action::new(
            "Copy parent path",
            ActionData::CopyToClipboard {
                text: parent.to_string_lossy().into_owned(),
            },
        ));
    }
    actions
}

fn git_remote_https(value: &str) -> Option<String> {
    let (host, path) = value.strip_prefix("git@")?.split_once(':')?;
    if !host.contains('.')
        || !host
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
        || path.is_empty()
        || !path
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-/".contains(&c))
    {
        return None;
    }
    let repo = path.strip_suffix(".git").unwrap_or(path);
    if repo.is_empty() {
        return None;
    }
    Some(format!("https://{host}/{repo}"))
}

fn hex_color(value: &str) -> Option<(u8, u8, u8)> {
    let digits = value.strip_prefix('#')?;
    match digits.len() {
        3 if digits.bytes().all(|c| c.is_ascii_hexdigit()) => {
            let r = u8::from_str_radix(&digits[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&digits[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&digits[2..3], 16).ok()? * 17;
            Some((r, g, b))
        }
        6 if digits.bytes().all(|c| c.is_ascii_hexdigit()) => Some((
            u8::from_str_radix(&digits[0..2], 16).ok()?,
            u8::from_str_radix(&digits[2..4], 16).ok()?,
            u8::from_str_radix(&digits[4..6], 16).ok()?,
        )),
        _ => None,
    }
}

fn smart_actions(value: &str) -> Vec<Action> {
    static EMAIL: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(r"(?i)^[a-z0-9._%+-]+@[a-z0-9-]+(?:\.[a-z0-9-]+)+$").unwrap()
    });
    static RGB: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(r"(?i)^rgb\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$").unwrap()
    });

    let value = value.trim();
    let mut actions = Vec::new();

    if let Ok(mut url) = reqwest::Url::parse(value) {
        if matches!(url.scheme(), "http" | "https") {
            actions.push(Action::new(
                "Open in browser",
                ActionData::OpenUrl { url: value.into() },
            ));
            if let Some(host) = url.host_str() {
                actions.push(Action::new(
                    "Copy domain",
                    ActionData::CopyToClipboard { text: host.into() },
                ));
            }
            if url.query().is_some() || url.fragment().is_some() {
                url.set_query(None);
                url.set_fragment(None);
                actions.push(Action::new(
                    "Copy URL without query or fragment",
                    ActionData::CopyToClipboard { text: url.into() },
                ));
            }
        }
    } else if let Some(https) = git_remote_https(value) {
        actions.push(Action::new(
            "Open repository",
            ActionData::OpenUrl { url: https.clone() },
        ));
        actions.push(Action::new(
            "Copy HTTPS URL",
            ActionData::CopyToClipboard { text: https },
        ));
    } else if EMAIL.is_match(value) {
        actions.push(Action::new(
            "Compose email",
            ActionData::OpenUrl {
                url: format!("mailto:{value}"),
            },
        ));
        if let Some((_, domain)) = value.rsplit_once('@') {
            actions.push(Action::new(
                "Copy domain",
                ActionData::CopyToClipboard {
                    text: domain.into(),
                },
            ));
        }
    }

    if matches!(value.as_bytes().first(), Some(b'{' | b'[')) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                if pretty != value {
                    actions.push(Action::new(
                        "Copy formatted JSON",
                        ActionData::CopyToClipboard { text: pretty },
                    ));
                }
            }
            if let Ok(compact) = serde_json::to_string(&json) {
                if compact != value {
                    actions.push(Action::new(
                        "Copy compact JSON",
                        ActionData::CopyToClipboard { text: compact },
                    ));
                }
            }
        }
    }

    if let Some((r, g, b)) = hex_color(value) {
        actions.push(Action::new(
            "Copy RGB",
            ActionData::CopyToClipboard {
                text: format!("rgb({r}, {g}, {b})"),
            },
        ));
        if value.len() == 4 {
            actions.push(Action::new(
                "Copy full HEX",
                ActionData::CopyToClipboard {
                    text: format!("#{r:02X}{g:02X}{b:02X}"),
                },
            ));
        }
    } else if let Some(caps) = RGB.captures(value) {
        let channels = (1..=3)
            .map(|i| caps[i].parse::<u8>())
            .collect::<Result<Vec<_>, _>>();
        if let Ok(channels) = channels {
            actions.push(Action::new(
                "Copy HEX",
                ActionData::CopyToClipboard {
                    text: format!("#{:02X}{:02X}{:02X}", channels[0], channels[1], channels[2]),
                },
            ));
        }
    }

    actions
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
        actions.push(Action::new(
            "Uppercase",
            ActionData::CopyToClipboard { text: upper },
        ));
    }

    let lower = value.to_lowercase();
    if lower != value {
        actions.push(Action::new(
            "Lowercase",
            ActionData::CopyToClipboard { text: lower },
        ));
    }

    let trimmed = value.trim().to_string();
    if trimmed != value {
        actions.push(Action::new(
            "Trim whitespace",
            ActionData::CopyToClipboard { text: trimmed },
        ));
    }

    let titled = title_case(value);
    if titled != value {
        actions.push(Action::new(
            "Title Case",
            ActionData::CopyToClipboard { text: titled },
        ));
    }

    let encoded = urlencoding::encode(value).to_string();
    if encoded != value {
        actions.push(Action::new(
            "URL encode",
            ActionData::CopyToClipboard { text: encoded },
        ));
    }

    if let Ok(decoded) = urlencoding::decode(value) {
        let decoded = decoded.into_owned();
        if decoded != value {
            actions.push(Action::new(
                "URL decode",
                ActionData::CopyToClipboard { text: decoded },
            ));
        }
    }

    let b64 = general_purpose::STANDARD.encode(value.as_bytes());
    actions.push(Action::new(
        "Base64 encode",
        ActionData::CopyToClipboard { text: b64 },
    ));

    if let Ok(decoded) = general_purpose::STANDARD.decode(value.trim().as_bytes()) {
        if let Ok(s) = String::from_utf8(decoded) {
            actions.push(Action::new(
                "Base64 decode",
                ActionData::CopyToClipboard { text: s },
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_actions_match_clipboard_content() {
        let git = smart_actions("git@github.com:veersheth/quarry.git");
        assert_eq!(
            git.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            ["Open repository", "Copy HTTPS URL"]
        );
        assert!(
            matches!(&git[0].data, ActionData::OpenUrl { url } if url == "https://github.com/veersheth/quarry")
        );

        let email = smart_actions("person@example.com");
        assert_eq!(
            email.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            ["Compose email", "Copy domain"]
        );
        assert!(
            matches!(&email[0].data, ActionData::OpenUrl { url } if url == "mailto:person@example.com")
        );

        let url = smart_actions("https://example.com/page?utm_source=feed#section");
        assert!(
            matches!(&url[2].data, ActionData::CopyToClipboard { text } if text == "https://example.com/page")
        );

        let json = smart_actions("{\"a\":1}");
        assert!(json.iter().any(|a| a.name == "Copy formatted JSON"));

        let color = smart_actions("#abc");
        assert!(
            matches!(&color[0].data, ActionData::CopyToClipboard { text } if text == "rgb(170, 187, 204)")
        );
        assert!(
            matches!(&color[1].data, ActionData::CopyToClipboard { text } if text == "#AABBCC")
        );
    }
}
