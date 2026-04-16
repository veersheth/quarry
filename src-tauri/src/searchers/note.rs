use crate::searchers::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct NoteSearcher;

impl SearchProvider for NoteSearcher {
    fn search(&self, _query: &str, _app: &tauri::AppHandle) -> SearchResult {
        SearchResult {
            results: vec![
                ResultItem::new("Scratch Note", vec![ActionData::RunFunction {
                    function_name: "open_note".into(),
                    params: vec![],
                }])
                .description("Open floating scratch note"),
            ],
            result_type: ResultType::List,
        }
    }
}
