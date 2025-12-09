use tauri::AppHandle;
use super::super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;

pub struct GitHubSearcher;

impl SearchProvider for GitHubSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        
        let url = format!( "https://www.github.com/search?q={}", urlencoding::encode(q));
        let action_id = format!("search_{}", url);
        
        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::OpenUrl { url: url.clone() }
            );
        }
        
        let results = vec![ResultItem {
            name: format!("Search GitHub for '{}'", q),
            action_id,
            description: Some("Open in browser".into()),
            icon: Some("https://upload.wikimedia.org/wikipedia/commons/thumb/a/ae/Github-desktop-logo-symbol.svg/2048px-Github-desktop-logo-symbol.svg.png".into()),
        }];
        
        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}

