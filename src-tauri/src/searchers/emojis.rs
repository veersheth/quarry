use once_cell::sync::Lazy;
use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct EmojiSearcher;

// ---------------------------------------------------------------------------
// Caches — built once at startup, cloned per search call
// ---------------------------------------------------------------------------

static ALL_EMOJIS: Lazy<Vec<ResultItem>> = Lazy::new(|| {
    emojis::iter().map(|e| make_emoji_item(e, false)).collect()
});

static ALL_SYMBOLS: Lazy<Vec<ResultItem>> = Lazy::new(|| {
    SYMBOL_RANGES.iter()
        .flat_map(|r| r.clone())
        .filter_map(char::from_u32)
        .filter(|&c| EmojiSearcher::is_symbol(c))
        .filter_map(EmojiSearcher::symbol_item)
        .collect()
});

/// Call at startup to initialise both caches in a background thread.
pub fn warm() {
    let _ = ALL_EMOJIS.len();
    let _ = ALL_SYMBOLS.len();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl EmojiSearcher {
    pub(super) fn format_unicode(emoji: &emojis::Emoji) -> String {
        emoji.as_str().chars()
            .map(|c| format!("U+{:X}", c as u32))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn format_unicode_char(c: char) -> String {
        format!("U+{:04X}", c as u32)
    }

    pub(super) fn is_symbol(c: char) -> bool {
        if c.is_ascii() { return false; }
        matches!(
            unicode_general_category::get_general_category(c),
            unicode_general_category::GeneralCategory::MathSymbol
                | unicode_general_category::GeneralCategory::CurrencySymbol
                | unicode_general_category::GeneralCategory::ModifierSymbol
                | unicode_general_category::GeneralCategory::OtherSymbol
        )
    }

    pub(super) fn symbol_item(c: char) -> Option<ResultItem> {
        let name = unicode_names2::name(c).map(|n| {
            let s = n.to_string();
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })?;

        let symbol_str = c.to_string();
        let unicode_str = Self::format_unicode_char(c);

        Some(ResultItem::new(
            symbol_str,
            vec![
                Action::new("Copy Symbol", ActionData::CopyToClipboard { text: c.to_string() }),
                Action::new("Copy Name",   ActionData::CopyToClipboard { text: name.clone() }),
                Action::new("Copy Unicode Hex", ActionData::CopyToClipboard { text: unicode_str }),
            ],
        ).description(name))
    }
}

const SYMBOL_RANGES: &[std::ops::RangeInclusive<u32>] = &[
    0x00A0..=0x00FF, 0x0370..=0x03FF, 0x2000..=0x206F, 0x2070..=0x209F,
    0x20A0..=0x20CF, 0x2100..=0x214F, 0x2150..=0x218F, 0x2190..=0x21FF,
    0x2200..=0x22FF, 0x2300..=0x23FF, 0x2400..=0x243F, 0x2440..=0x245F,
    0x2460..=0x24FF, 0x2500..=0x257F, 0x2580..=0x259F, 0x25A0..=0x25FF,
    0x2600..=0x26FF, 0x2700..=0x27BF, 0x27C0..=0x27EF, 0x27F0..=0x27FF,
    0x2900..=0x297F, 0x2980..=0x29FF, 0x2A00..=0x2AFF, 0x2B00..=0x2BFF,
];

fn make_emoji_item(emoji: &emojis::Emoji, is_pinned: bool) -> ResultItem {
    let emoji_str = emoji.as_str().to_string();
    let name = emoji.name().to_string();

    let pin_action = if is_pinned {
        Action::new("Unpin", ActionData::RunFunction {
            function_name: "unpin".into(),
            params: vec!["emojis".into(), emoji_str.clone()],
        })
    } else {
        Action::new("Pin", ActionData::RunFunction {
            function_name: "pin".into(),
            params: vec!["emojis".into(), emoji_str.clone(), emoji_str.clone()],
        })
    };

    let mut actions = vec![
        Action::new("Copy Emoji", ActionData::CopyToClipboard { text: emoji_str.clone() }),
        pin_action,
        Action::new("Copy Name", ActionData::CopyToClipboard { text: name.clone() }),
        Action::new("Copy Unicode Hex", ActionData::CopyToClipboard {
            text: EmojiSearcher::format_unicode(emoji),
        }),
    ];
    if let Some(shortcode) = emoji.shortcode() {
        actions.push(Action::new("Copy Shortcode", ActionData::CopyToClipboard {
            text: format!(":{}:", shortcode),
        }));
    }

    let mut item = ResultItem::new(emoji_str, actions).description(name);
    if is_pinned { item = item.pinned(); }
    item
}

// ---------------------------------------------------------------------------
// SearchProvider
// ---------------------------------------------------------------------------

impl SearchProvider for EmojiSearcher {
    fn name(&self) -> String { "emojis".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim().to_lowercase();

        if q.is_empty() {
            let pins = crate::PINS.get("emojis");

            // 1. Pinned emojis (fresh, so Unpin action is correct)
            let pinned_items: Vec<ResultItem> = pins.iter()
                .filter_map(|pin| emojis::get(&pin.payload))
                .map(|emoji| make_emoji_item(emoji, true).group("featured"))
                .collect();

            // 2. 10 recent emojis (fresh, small set)
            let recent_items: Vec<ResultItem> = crate::usage_tracker::get_recent_entries("", 10)
                .into_iter()
                .filter_map(|e| emojis::get(&e.name))
                .map(|emoji| make_emoji_item(emoji, false).group("featured"))
                .collect();

            // 3. All emojis then symbols — served from cache
            let results = pinned_items.into_iter()
                .chain(recent_items)
                .chain(ALL_EMOJIS.iter().cloned())
                .chain(ALL_SYMBOLS.iter().cloned())
                .collect();

            return SearchResult { results, result_type: ResultType::Grid, ..Default::default() };
        }

        // Non-empty query: filter emojis via emojis::iter() (static, supports shortcodes),
        // filter symbols from the cache to avoid re-doing unicode_names2 lookups.
        let emoji_results = emojis::iter()
            .filter(|emoji| {
                emoji.name().to_lowercase().contains(&q)
                    || emoji.shortcode().map_or(false, |s| s.contains(&q))
            })
            .map(|emoji| make_emoji_item(emoji, false));

        let symbol_results = ALL_SYMBOLS.iter()
            .filter(|item| {
                item.description.as_deref()
                    .map_or(false, |d| d.to_lowercase().contains(&q))
                    || item.name == q
            })
            .cloned();

        let results = emoji_results.chain(symbol_results).collect();
        SearchResult { results, result_type: ResultType::Grid, ..Default::default() }
    }
}
