use crate::searchers::SearchProvider;
use crate::types::{ResultItem, SearchResult};

pub struct CameraSearcher;

impl SearchProvider for CameraSearcher {
    fn search(&self, _query: &str, _app: &tauri::AppHandle) -> SearchResult {
        SearchResult {
            results: vec![ResultItem {
                name: "Camera".to_string(),
                action_id: "camera_preview".to_string(),
                description: Some("Live camera preview".to_string()),
                icon: None,
            }],
            result_type: crate::types::ResultType::Camera
        }
    }
}
