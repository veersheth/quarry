use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;

pub struct ColorPicker;

impl SearchProvider for ColorPicker {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        SearchResult {
            results: vec![],
            result_type: ResultType::ColorPicker,
        }
    }
}
