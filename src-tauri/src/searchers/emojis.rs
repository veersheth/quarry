use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct EmojiSearcher;

impl SearchProvider for EmojiSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim().to_lowercase();

        let results = emojis::iter()
            .filter(|emoji| {
                q.is_empty()
                    || emoji.name().to_lowercase().contains(&q)
                    || emoji.shortcode().map_or(false, |s| s.contains(&q))
            })
            .take(50)
            .map(|emoji| {
                ResultItem::new(
                    emoji.as_str(),
                    ActionData::CopyToClipboard { text: emoji.as_str().to_string() },
                )
                .description(emoji.name())
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::Grid,
        }
    }
}
