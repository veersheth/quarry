use super::super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use tauri::AppHandle;

pub struct URLSearcher;

fn looks_like_url(q: &str) -> bool {
    if q.starts_with("http://") || q.starts_with("https://") {
        return true;
    }

    // eg example.com localhost:3000 192.168.1.1
    let without_path = q.split('/').next().unwrap_or("");
    let without_port = without_path.split(':').next().unwrap_or("");
    without_port.contains('.') || without_port == "localhost"
}

impl SearchProvider for URLSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        if q.is_empty() || !looks_like_url(q) {
            return SearchResult {
                results: vec![],
                result_type: ResultType::WebSearch,
            };
        }

        // prepend https
        let url = if q.starts_with("http://") || q.starts_with("https://") {
            q.to_string()
        } else {
            format!("https://{}", q)
        };

        let results = vec![
            ResultItem::new(
                format!("Open '{}'", q),
                ActionData::OpenUrl { url },
            )
            .description("Open URL")
            .icon(""),
        ];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}
