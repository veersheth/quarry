use super::super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use tauri::AppHandle;

pub struct YouTubeSearcher;

impl SearchProvider for YouTubeSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let url = format!(
            "https://www.youtube.com/results?search_query={}",
            urlencoding::encode(q)
        );

        let results = vec![
            ResultItem::new(
                format!("Search YouTube for '{}'", q),
                ActionData::OpenUrl { url },
            )
            .description("Open in browser")
            .icon("https://upload.wikimedia.org/wikipedia/commons/thumb/f/fd/YouTube_full-color_icon_%282024%29.svg/1920px-YouTube_full-color_icon_%282024%29.svg.png"),
        ];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}
