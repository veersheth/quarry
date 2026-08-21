use crate::searchers::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct QrCodeSearcher;

impl SearchProvider for QrCodeSearcher {
    fn name(&self) -> String { "qr".to_string() }

    fn search(&self, query: &str, _app: &tauri::AppHandle) -> SearchResult {
        // The text is passed as-is; QR generation happens on the frontend.
        // No background work, no network, nothing runs until triggered.
        let text = query.trim();
        SearchResult {
            results: vec![
                ResultItem::new(text, vec![Action::new("Copy", ActionData::CopyToClipboard { text: text.into() })])
                    .description("QR code"),
            ],
            result_type: ResultType::QrCode,
            ..Default::default()
        }
    }
}
