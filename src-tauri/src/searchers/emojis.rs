use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct EmojiSearcher;

impl EmojiSearcher {
    fn format_unicode(emoji: &emojis::Emoji) -> String {
        emoji
            .as_str()
            .chars()
            .map(|c| format!("U+{:X}", c as u32))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn format_unicode_char(c: char) -> String {
        format!("U+{:04X}", c as u32)
    }

    fn is_symbol(c: char) -> bool {
        if c.is_ascii() {
            return false;
        }
        matches!(
            unicode_general_category::get_general_category(c),
            unicode_general_category::GeneralCategory::MathSymbol
                | unicode_general_category::GeneralCategory::CurrencySymbol
                | unicode_general_category::GeneralCategory::ModifierSymbol
                | unicode_general_category::GeneralCategory::OtherSymbol
        )
    }
}

impl SearchProvider for EmojiSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim().to_lowercase();

        let emoji_results = emojis::iter()
            .filter(|emoji| {
                q.is_empty()
                    || emoji.name().to_lowercase().contains(&q)
                    || emoji.shortcode().map_or(false, |s| s.contains(&q))
            })
            .map(|emoji| {
                let emoji_str = emoji.as_str().to_string();
                let name = emoji.name().to_string();

                let mut actions = vec![
                    Action::new(
                        "Copy Emoji",
                        ActionData::CopyToClipboard { text: emoji_str.clone() },
                    ),
                    Action::new(
                        "Copy Name",
                        ActionData::CopyToClipboard { text: name.clone() },
                    ),
                    Action::new(
                        "Copy Unicode Hex",
                        ActionData::CopyToClipboard {
                            text: Self::format_unicode(emoji),
                        },
                    ),
                ];
                if let Some(shortcode) = emoji.shortcode() {
                    actions.push(Action::new(
                        "Copy Shortcode",
                        ActionData::CopyToClipboard {
                            text: format!(":{}:", shortcode),
                        },
                    ));
                }
                ResultItem::new(emoji_str, actions).description(name)
            });

        let symbol_ranges: &[std::ops::RangeInclusive<u32>] = &[
            0x00A0..=0x00FF,
            0x0370..=0x03FF,
            0x2000..=0x206F,
            0x2070..=0x209F,
            0x20A0..=0x20CF,
            0x2100..=0x214F,
            0x2150..=0x218F,
            0x2190..=0x21FF,
            0x2200..=0x22FF,
            0x2300..=0x23FF,
            0x2400..=0x243F,
            0x2440..=0x245F,
            0x2460..=0x24FF,
            0x2500..=0x257F,
            0x2580..=0x259F,
            0x25A0..=0x25FF,
            0x2600..=0x26FF,
            0x2700..=0x27BF,
            0x27C0..=0x27EF,
            0x27F0..=0x27FF,
            0x2900..=0x297F,
            0x2980..=0x29FF,
            0x2A00..=0x2AFF,
            0x2B00..=0x2BFF,
        ];

        let symbol_results = symbol_ranges
            .iter()
            .flat_map(|range| range.clone())
            .filter_map(|cp| char::from_u32(cp))
            .filter(|&c| Self::is_symbol(c))
            .filter(|&c| {
                if q.is_empty() {
                    return true;
                }
                unicode_names2::name(c)
                    .map_or(false, |n| n.to_string().to_lowercase().contains(&q))
                    || c.to_string() == q
            })
            .filter_map(|c| {
                let name = unicode_names2::name(c)
                    .map(|n| {
                        let s = n.to_string();
                        let mut chars = s.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>()
                                    + &chars.as_str().to_lowercase()
                            }
                        }
                    })?;

                let symbol_str = c.to_string();
                let unicode_str = Self::format_unicode_char(c);

                let actions = vec![
                    Action::new(
                        "Copy Symbol",
                        ActionData::CopyToClipboard { text: symbol_str.clone() },
                    ),
                    Action::new(
                        "Copy Name",
                        ActionData::CopyToClipboard { text: name.clone() },
                    ),
                    Action::new(
                        "Copy Unicode Hex",
                        ActionData::CopyToClipboard { text: unicode_str },
                    ),
                ];

                Some(ResultItem::new(symbol_str, actions).description(name))
            });

        let results = emoji_results.chain(symbol_results).collect();

        SearchResult {
            results,
            result_type: ResultType::Grid,
        }
    }
}
