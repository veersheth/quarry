use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::CLIPBOARD_MANAGER;
use tauri::AppHandle;

pub struct ClipboardSearcher;

impl SearchProvider for ClipboardSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim().to_lowercase();

        if query == "clear" {
            return SearchResult {
                results: vec![
                    ResultItem::new("Clear clipboard history?", ActionData::RunFunction {
                        function_name: "clear_clipboard".into(),
                        params: vec![],
                    }),
                ],
                result_type: ResultType::List,
            };
        }

        let history = CLIPBOARD_MANAGER.get_history();

        let results = history
            .iter()
            .filter(|entry| {
                query.is_empty() || entry.content.to_lowercase().contains(&query)
            })
            .map(|entry| {
                ResultItem::new(
                    entry.content.clone(),
                    ActionData::CopyToClipboard { text: entry.content.clone() },
                )
                .description(entry.timestamp.to_string())
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::Clipboard,
        }
    }
}
