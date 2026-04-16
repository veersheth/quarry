use crate::searchers::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct CameraSearcher;

impl SearchProvider for CameraSearcher {
    fn search(&self, _query: &str, _app: &tauri::AppHandle) -> SearchResult {
        SearchResult {
            results: vec![
                ResultItem::new("Camera", vec![ActionData::None])
                    .description("Live camera preview"),
            ],
            result_type: ResultType::Camera,
        }
    }
}
