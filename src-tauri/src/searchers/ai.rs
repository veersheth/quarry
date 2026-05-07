use crate::searchers::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use tauri::AppHandle;

pub struct AiSearcher;

impl SearchProvider for AiSearcher {
    fn name(&self) -> String { "ai".to_string() }
    fn search(&self, _query: &str, _app: &AppHandle) -> SearchResult {
        // the frontend renders the full chat ui inline and reads the
        // query directly from the store - no result items needed.
        SearchResult {
            results: vec![],
            result_type: ResultType::Ai,
                    ..Default::default()
}
    }
}
