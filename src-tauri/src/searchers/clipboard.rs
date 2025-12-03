use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::{ACTION_REGISTRY, CLIPBOARD_MANAGER};
use tauri::AppHandle;

pub struct ClipboardSearcher;

impl SearchProvider for ClipboardSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim().to_lowercase();

        if query == "!clear" {
            let action_id = "clipboard_clear".to_string();

            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::RunFunction {
                        function_name: "clear_clipboard".into(),
                        params: vec![],
                    },
                );
            }

            return SearchResult {
                results: vec![ResultItem {
                    name: "Clear clipboard history?".into(),
                    action_id,
                    description: None,
                    icon: None,
                }],
                result_type: ResultType::List, 
            };
        }

        let history = CLIPBOARD_MANAGER.get_history();

        let results: Vec<ResultItem> = history
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                if query.is_empty() {
                    true
                } else {
                    entry.content.to_lowercase().contains(&query)
                }
            })
            .map(|(idx, entry)| {
                let action_id = format!("clipboard_{}", idx);

                if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                    registry.register(
                        action_id.clone(),
                        ActionData::CopyToClipboard {
                            text: entry.content.clone(),
                        },
                    );
                }

                ResultItem {
                    name: entry.content.clone(),
                    action_id,
                    description: None,
                    icon: None,
                }
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::Clipboard,
        }
    }
}
