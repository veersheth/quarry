use super::super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::ACTION_REGISTRY;
use tauri::AppHandle;

pub struct YouTubeSearcher;

impl SearchProvider for YouTubeSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let url = format!(
            "https://www.youtube.com/results?search_query={}",
            urlencoding::encode(q)
        );
        let action_id = format!("search_{}", url);

        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(action_id.clone(), ActionData::OpenUrl { url: url.clone() });
        }

        let results = vec![ResultItem {
            name: format!("Search YouTube for '{}'", q),
            action_id,
            description: Some("Open in browser".into()),
            icon: Some("https://upload.wikimedia.org/wikipedia/commons/thumb/0/09/YouTube_full-color_icon_%282017%29.svg/2560px-YouTube_full-color_icon_%282017%29.svg.png".into()),
        }];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}
