use super::SearchProvider;
use crate::clipboard_manager::ClipboardContent;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use crate::CLIPBOARD_MANAGER;
use tauri::AppHandle;

pub struct ClipboardSearcher;

impl SearchProvider for ClipboardSearcher {
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
            };
        }

        let text_pins = crate::PINS.get("clipboard");
        let pinned_texts: std::collections::HashSet<&str> =
            text_pins.iter().map(|p| p.payload.as_str()).collect();

        let image_pins = crate::PINS.get("clipboard_image");
        let pinned_hashes: std::collections::HashSet<u64> = image_pins
            .iter()
            .filter_map(|p| p.name.parse::<u64>().ok())
            .collect();

        let history = CLIPBOARD_MANAGER.get_history();

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

        let history_items = history
            .iter()
            .filter(|entry| {
                if query.is_empty() {
                    return match &entry.content {
                        ClipboardContent::Text { value } => !pinned_texts.contains(value.as_str()),
                        ClipboardContent::Image { hash, .. } => !pinned_hashes.contains(hash),
                    };
                }
                entry.display_text().to_lowercase().contains(&query)
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
                    ResultItem::new(
                        value.clone(),
                        vec![
                            Action::new("Copy", ActionData::CopyToClipboard { text: value.clone() }),
                            pin_action,
                            Action::new("Delete", ActionData::RunFunction {
                                function_name: "delete_clipboard_entry".into(),
                                params: vec![entry.timestamp.to_string()],
                            }),
                            Action::new("Clear entire clipboard", ActionData::RunFunction {
                                function_name: "clear_clipboard".into(),
                                params: vec![],
                            }),
                        ],
                    )
                    .description(format_timestamp(entry.timestamp))
                }

                ClipboardContent::Image { thumbnail, full, width, height, ocr_text, hash } => {
                    let hash_str = hash.to_string();
                    let is_pinned = crate::PINS.contains("clipboard_image", &hash_str);
                    let pin_action = if is_pinned {
                        Action::new("Unpin", ActionData::RunFunction {
                            function_name: "unpin".into(),
                            params: vec!["clipboard_image".into(), hash_str],
                        })
                    } else {
                        let payload = serde_json::json!({
                            "full": full,
                            "thumbnail": thumbnail,
                            "width": width,
                            "height": height,
                        })
                        .to_string();
                        Action::new("Pin", ActionData::RunFunction {
                            function_name: "pin".into(),
                            params: vec!["clipboard_image".into(), hash_str, payload],
                        })
                    };

                    let mut actions = vec![
                        Action::new("Copy", ActionData::CopyImageToClipboard {
                            base64_png: full.clone(),
                            width: *width,
                            height: *height,
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
            });

        results.extend(history_items);

        SearchResult {
            results,
            result_type: ResultType::Clipboard,
        }
    }
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
