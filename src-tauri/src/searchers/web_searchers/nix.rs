use super::super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::ACTION_REGISTRY;
use tauri::AppHandle;

pub struct NixSearcher;

impl SearchProvider for NixSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let url = format!(
            "https://search.nixos.org/packages?channel=25.05&query={}",
            urlencoding::encode(q)
        );
        let action_id = format!("search_{}", url);

        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(action_id.clone(), ActionData::OpenUrl { url: url.clone() });
        }

        let results = vec![ResultItem {
            name: format!("Search Nix Packages for '{}'", q),
            action_id,
            description: Some("Open in browser".into()),
            icon: Some(
                "https://upload.wikimedia.org/wikipedia/commons/3/35/Nix_Snowflake_Logo.svg".into(),
            ),
        }];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
            usage_sorted: false,
            additional_info: None,
        }
    }
}
