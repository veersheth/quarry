use tauri::AppHandle;
use super::super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};

pub struct NixSearcher;

impl SearchProvider for NixSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        
        let url = format!( "https://search.nixos.org/packages?query={}", urlencoding::encode(q));
        
        let results = vec![
            ResultItem::new(
                format!("Search Nix Packages for '{}'", q),
                ActionData::OpenUrl { url },
            )
            .description("Open in browser")
            .icon("https://upload.wikimedia.org/wikipedia/commons/thumb/0/09/YouTube_full-color_icon_%282017%29.svg/2560px-YouTube_full-color_icon_%282017%29.svg.png"),
        ];

        SearchResult {
            results,
            result_type: ResultType::WebSearch,
        }
    }
}

