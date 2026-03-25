pub mod default;
pub mod apps;
pub mod emojis;
pub mod lorem;
pub mod math;
pub mod shell;
pub mod dictionary;
pub mod system;
pub mod web_searchers;
pub mod clipboard;
pub mod colorpicker;
pub mod files;
pub mod bookmarks;
pub mod camera;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tauri::AppHandle;
use crate::types::{ResultItem, SearchResult};

pub trait SearchProvider {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult;

    fn fuzzy_filter(&self, mut items: Vec<ResultItem>, query: &str) -> Vec<ResultItem> {
        if query.is_empty() {
            return items;
        }

        let matcher = SkimMatcherV2::default();

        let mut scored: Vec<(ResultItem, i64)> = items
            .into_iter()
            .filter_map(|item| {
                let score = fuzzy_score(&matcher, &item, query);
                if score > 0 { Some((item, score)) } else { None }
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(item, _)| item).collect()
    }
}

fn fuzzy_score(matcher: &SkimMatcherV2, item: &ResultItem, query: &str) -> i64 {
    let name_score = matcher
        .fuzzy_match(&item.name, query)
        .map(|s| s * 3)          // name
        .unwrap_or(0);

    let desc_score = item.description
        .as_deref()
        .and_then(|d| matcher.fuzzy_match(d, query))
        .map(|s| s * 2)          // description
        .unwrap_or(0);

    let id_score = matcher
        .fuzzy_match(&item.action_id, query)
        .unwrap_or(0);           // action id

    name_score.max(desc_score).max(id_score)
}

