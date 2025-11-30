use crate::searchers::SearchProvider;
use crate::types::{SearchResult, ResultItem, ResultType};

pub struct ShellSearcher;

impl SearchProvider for ShellSearcher {
    fn search(&self, query: &str) -> SearchResult {
        let command = query.trim();

        let results = vec![ResultItem {
            name: command.to_string(),
            exec: Some(format!("sh -c '{}'", command.replace('\'', "'\\''"))),
            description: Some("Run shell command".into()),
            icon: None,
        }];

        SearchResult { 
            results, 
            result_type: ResultType::List 
        }
    }
}

