use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct EmojiSearcher;

impl EmojiSearcher {
    /// Formats the unicode scalar values to a string like "U+1F600"
    fn format_unicode(emoji: &emojis::Emoji) -> String {
        emoji
            .as_str()
            .chars()
            .map(|c| format!("U+{:X}", c as u32))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

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
                let emoji_str = emoji.as_str().to_string();
                let name = emoji.name().to_string();
                
                let mut actions = vec![
                    Action::new(
                        "Copy Emoji", 
                        ActionData::CopyToClipboard { text: emoji_str.clone() }
                    ),
                    Action::new(
                        "Copy Name", 
                        ActionData::CopyToClipboard { text: name.clone() }
                    ),
                    Action::new(
                        "Copy Unicode Hex", 
                        ActionData::CopyToClipboard { text: Self::format_unicode(emoji) }
                    ),
                ];

                if let Some(shortcode) = emoji.shortcode() {
                    actions.push(Action::new(
                        "Copy Shortcode",
                        ActionData::CopyToClipboard { text: format!(":{}:", shortcode) }
                    ));
                }

                ResultItem::new(emoji_str, actions)
                    .description(name)
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::Grid,
        }
    }
}
