use crate::searchers::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use tauri::AppHandle;

pub struct AiSearcher;

impl SearchProvider for AiSearcher {
    fn name(&self) -> String { "ai".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let key = crate::config::Config::load_groq_api_key();
        if !key.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::Ai,
                ..Default::default()
            };
        }

        // No key - let the user paste it directly into the query.
        let q = query.trim();
        let item = if q.is_empty() {
            ResultItem::new(
                "Groq API key required",
                vec![],
            )
            .description("Type your Groq API key and press Enter to save it")
            .icon("icons/settings.png")
        } else {
            ResultItem::new(
                "Save Groq API key",
                vec![Action::new(
                    "Save",
                    ActionData::RunFunction {
                        function_name: "save_groq_api_key".into(),
                        params: vec![q.to_string()],
                    },
                )],
            )
            .description(format!("Save '{}' as your Groq API key", q))
            .icon("icons/settings.png")
        };

        SearchResult {
            results: vec![item],
            result_type: ResultType::List,
            ..Default::default()
        }
    }
}
