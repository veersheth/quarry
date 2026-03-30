use super::super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use tauri::AppHandle;

pub struct GoogleSearcher;

impl SearchProvider for GoogleSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let url = format!("https://www.google.com/search?q={}", urlencoding::encode(q));

        let results = vec![
            ResultItem::new(
                format!("Search Google for '{}'", q),
                ActionData::OpenUrl { url },
            )
            .description("Open in browser")
            .icon("https://upload.wikimedia.org/wikipedia/commons/thumb/c/c1/Google_%22G%22_logo.svg/1200px-Google_%22G%22_logo.svg.png"),
        ];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}
