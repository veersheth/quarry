use super::super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult};

pub struct NixSearcher;

impl SearchProvider for NixSearcher {
    fn search(&self, query: &str) -> SearchResult {
        let q = query.trim();
        if q.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
            };
        }

        let url = format!(
            "https://search.nixos.org/packages?channel=25.05&query={}",
            urlencoding::encode(q)
        );

        let results = vec![ResultItem {
            name: format!("Search Nix Packages for '{}'", q),
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
