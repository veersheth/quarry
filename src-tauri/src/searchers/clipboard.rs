use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::{ACTION_REGISTRY, CLIPBOARD_MANAGER};

pub struct ClipboardSearcher;

impl SearchProvider for ClipboardSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let history = CLIPBOARD_MANAGER.get_history();
        
        let query_lower = query.trim().to_lowercase();
        
        let results: Vec<ResultItem> = history
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                if query_lower.is_empty() {
                    true
                } else {
                    entry.content.to_lowercase().contains(&query_lower)
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
                    name: entry.preview.clone(),
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
