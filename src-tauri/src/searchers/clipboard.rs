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

        let history = CLIPBOARD_MANAGER.get_history();
        let results = history
            .iter()
            .filter(|entry| {
                if query.is_empty() {
                    return true;
                }
                entry
                    .display_text()
                    .to_lowercase()
                    .contains(&query)
            })
            .map(|entry| match &entry.content {
                ClipboardContent::Text { value } => ResultItem::new(
                    value.clone(),
                    vec![
                        Action::new("Copy", ActionData::CopyToClipboard { text: value.clone() }),
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
                .description(format_timestamp(entry.timestamp)),

                ClipboardContent::Image { thumbnail, full, width, height, .. } => ResultItem::new(
                    format!("Image {}×{}", width, height),
                    vec![
                        Action::new("Copy", ActionData::CopyImageToClipboard {
                            base64_png: full.clone(),
                            width: *width,
                            height: *height,
                        }),
                        Action::new("Delete", ActionData::RunFunction {
                            function_name: "delete_clipboard_entry".into(),
                            params: vec![entry.timestamp.to_string()],
                        }),
                    ],
                )
                .description(format_timestamp(entry.timestamp))
                .thumbnail(thumbnail.clone()),
            })
            .collect();

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
