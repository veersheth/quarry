use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult};

pub struct ColorPicker;

impl SearchProvider for ColorPicker {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        // Pass the raw color string as the first result's name so the
        // frontend component can parse and pre-initialise the picker.
        let results = if q.is_empty() {
            vec![]
        } else {
            vec![ResultItem::new(q, vec![])]
        };
        SearchResult { results, result_type: ResultType::ColorPicker }
    }
}
