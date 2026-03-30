use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultType, SearchResult};

pub struct ColorPicker;

impl SearchProvider for ColorPicker {
    fn search(&self, _query: &str, _app: &AppHandle) -> SearchResult {
        SearchResult {
            results: vec![],
            result_type: ResultType::ColorPicker,
        }
    }
}
