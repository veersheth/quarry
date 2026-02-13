use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::ACTION_REGISTRY;
use tauri::AppHandle;
use emojis;

pub struct EmojiSearcher;

impl SearchProvider for EmojiSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim().to_lowercase();
        
        let results: Vec<ResultItem> = emojis::iter()
            .filter(|emoji| {
                emoji.name().to_lowercase().contains(&q) ||
                emoji.shortcode().map_or(false, |s| s.contains(&q))
            })
            .map(|emoji| {
                let action_id = format!("emoji_{}", emoji.as_str());
                if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                    registry.register(
                        action_id.clone(),
                        ActionData::CopyToClipboard {
                            text: emoji.as_str().to_string(),
                        },
                    );
                }
                ResultItem {
                    name: emoji.as_str().to_string(),
                    action_id,
                    description: Some(emoji.name().to_string()),
                    icon: None,
                }
            })
            .collect();
        
        SearchResult {
            results,
            result_type: ResultType::Grid,
        }
    }
}
