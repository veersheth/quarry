use tauri::AppHandle;
use super::super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};

pub struct GitHubSearcher;

impl SearchProvider for GitHubSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        
        let url = format!( "https://www.github.com/search?q={}", urlencoding::encode(q));
        
        let results = vec![
            ResultItem::new(
                format!("Search GitHub for '{}'", q),
                ActionData::OpenUrl { url },
            )
            .description("Open in browser")
            .icon("https://upload.wikimedia.org/wikipedia/commons/thumb/a/ae/Github-desktop-logo-symbol.svg/2048px-Github-desktop-logo-symbol.svg.png"),
        ];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}

