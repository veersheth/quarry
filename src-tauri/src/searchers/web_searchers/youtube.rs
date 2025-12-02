use tauri::AppHandle;
use super::super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;

pub struct YouTubeSearcher;

impl SearchProvider for YouTubeSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        
        if q.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
            };
        }
        
        let url = format!( "https://www.youtube.com/results?search_query={}", urlencoding::encode(q));
        let action_id = format!("search_{}", url);
        
        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::OpenUrl { url: url.clone() }
            );
        }
        
        let results = vec![ResultItem {
            name: format!("Search YouTube for '{}'", q),
            action_id,
            description: Some("Open in browser".into()),
            icon: None,
        }];
        
        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}

