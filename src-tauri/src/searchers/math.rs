use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct MathSearcher;

impl SearchProvider for MathSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let expr = query.trim();

        let results = match meval::eval_str(expr) {
            Ok(result) => vec![
                ResultItem::new(
                    format!("{} = {}", expr, result),
                    ActionData::CopyToClipboard { text: result.to_string() },
                )
                .description("Copy result to clipboard")
                .icon("icons/math.png"),
            ],
            Err(_) => vec![],
        };

        SearchResult {
            results,
            result_type: ResultType::Math,
        }
    }
}
