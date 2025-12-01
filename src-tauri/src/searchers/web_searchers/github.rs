use super::super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult};

pub struct GithubSearcher;

impl SearchProvider for GithubSearcher {
    fn search(&self, query: &str) -> SearchResult {
        let q = query.trim();
        if q.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
            };
        }

        let url = format!("https://www.github.com/search?q={}", urlencoding::encode(q));

        let results = vec![ResultItem {
            name: format!("Search GitHub for '{}'", q),
            exec: Some(format!("xdg-open {url}")),
            description: None,
            icon: None,
        }];

        SearchResult {
            results: results,
            result_type: ResultType::List,
        }
    }
}
