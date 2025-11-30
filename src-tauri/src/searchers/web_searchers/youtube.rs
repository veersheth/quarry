use crate::types::{SearchResult, ResultItem, ResultType};
use super::super::SearchProvider;

pub struct YouTubeSearcher;

impl SearchProvider for YouTubeSearcher {
    fn search(&self, query: &str) -> SearchResult {
        let q = query.trim();
        if q.is_empty() {
            return SearchResult { results: vec![], result_type: ResultType::List };
        }

        let url = format!("https://www.youtube.com/search?q={}", urlencoding::encode(q));

        let results = vec![ResultItem {
            name: format!("Search YouTube for '{}'", q),
            exec: Some(format!("xdg-open {url}")),
            description: None,
            icon: None,
        }];

        SearchResult { results: results, result_type: ResultType::List }
    }
}

