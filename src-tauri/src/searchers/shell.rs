use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct ShellSearcher;

impl SearchProvider for ShellSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let cmd = query.trim();

        let results = if cmd.is_empty() {
            vec![]
        } else {
            vec![
                ResultItem::new(
                    format!("Run: {}", cmd),
                    vec![Action::new("Run", ActionData::ShellCommand { command: cmd.to_string() })],
                )
                .description("Execute shell command"),
            ]
        };

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
