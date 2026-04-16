use crate::searchers::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct CameraSearcher;

impl SearchProvider for CameraSearcher {
    fn search(&self, _query: &str, _app: &tauri::AppHandle) -> SearchResult {
        SearchResult {
            results: vec![
                ResultItem::new("Camera", vec![Action::new("Open", ActionData::None)])
                    .description("Live camera preview"),
            ],
            result_type: ResultType::Camera,
        }
    }
}
