use tauri::AppHandle;

use crate::searchers::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult};

pub struct MathSearcher;

impl SearchProvider for MathSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let expr = query.trim();

        let value = meval::eval_str(expr);
        let results = match value {
            Ok(result) => vec![ResultItem {
                name: format!("{} = {}", expr, result),
                exec: Some(format!("wl-copy {}", result)),
                description: Some("Copy to clipboard".into()),
                icon: None,
            }],
            Err(_) => vec![],
        };
        SearchResult {
            results: results,
            result_type: ResultType::List,
        }
    }
}
