use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::ACTION_REGISTRY;
use tauri::AppHandle;

pub struct ShellSearcher;

impl SearchProvider for ShellSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let cmd = query.trim();

        if cmd.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
                usage_sorted: false,
                additional_info: None,
            };
        }

        let action_id = format!("shell_{}", cmd);

        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::ShellCommand {
                    command: cmd.to_string(),
                },
            );
        }

        let results = vec![ResultItem {
            name: format!("Run: {}", cmd),
            action_id,
            description: Some("Execute shell command".into()),
            icon: None,
        }];

        SearchResult {
            results,
            result_type: ResultType::List,
            usage_sorted: false,
            additional_info: None,
        }
    }
}
