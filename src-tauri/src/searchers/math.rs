use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;

pub struct MathSearcher;

impl SearchProvider for MathSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let expr = query.trim();
        let value = meval::eval_str(expr);
        
        let results = match value {
            Ok(result) => {
                let result_str = result.to_string();
                let action_id = format!("math_{}", result_str);
                
                if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                    registry.register(
                        action_id.clone(),
                        ActionData::CopyToClipboard {
                            text: result_str.clone(),
                        },
                    );
                }
                
                vec![ResultItem {
                    name: format!("{} = {}", expr, result),
                    action_id,
                    description: Some("Copy result to clipboard".into()),
                    icon: None,
                }]
            }
            Err(_) => vec![],
        };
        
        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
