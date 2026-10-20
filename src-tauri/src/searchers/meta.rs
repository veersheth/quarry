use tauri::AppHandle;

use crate::types::{Action, ActionData, ResultItem, SearchResult};
use crate::CONFIG;

use super::SearchProvider;

struct SearchTarget {
    name: String,
    trigger: String,
}

pub struct MetaSearcher;

impl MetaSearcher {
    fn targets() -> Vec<SearchTarget> {
        let Ok(config) = CONFIG.read() else {
            return Vec::new();
        };

        let triggers = &config.triggers;
        let configured = [
            ("Apps", triggers.apps.as_str()),
            ("Files", triggers.files.as_str()),
            ("Clipboard", triggers.clipboard.as_str()),
            ("Emojis", triggers.emojis.as_str()),
            ("Fonts", triggers.fonts.as_str()),
            ("Bookmarks", triggers.bookmarks.as_str()),
            ("System", triggers.system.as_str()),
            ("Shortcuts", triggers.shortcuts.as_str()),
            ("Scripts", triggers.scripts.as_str()),
            ("Screenshots", triggers.screenshots.as_str()),
            ("Math", triggers.math.as_str()),
            ("Dictionary", triggers.dictionary.as_str()),
            ("Time", triggers.time.as_str()),
            ("Timer", triggers.timer.as_str()),
            ("AI", triggers.ai.as_str()),
            ("Camera", triggers.camera.as_str()),
            ("Color Picker", triggers.color_picker.as_str()),
            ("Currency", triggers.currency.as_str()),
            ("QR Code", triggers.qr_code.as_str()),
            ("Notes", triggers.note.as_str()),
        ];

        let mut targets: Vec<SearchTarget> = configured
            .into_iter()
            .filter_map(|(name, pattern)| {
                trigger_query(pattern).map(|trigger| SearchTarget {
                    name: name.to_string(),
                    trigger,
                })
            })
            .collect();

        targets.extend(config.web_searches.iter().filter_map(|web_search| {
            trigger_query(&web_search.trigger).map(|trigger| SearchTarget {
                name: format!("{} Search", web_search.name),
                trigger,
            })
        }));
        targets
    }

    fn search_targets(query: &str) -> Option<Vec<SearchTarget>> {
        let words: Vec<&str> = query.split_whitespace().collect();
        if !words
            .first()
            .is_some_and(|word| word.eq_ignore_ascii_case("search"))
        {
            return None;
        }

        let target_query = words
            .iter()
            .skip(1)
            .copied()
            .skip_while(|word| matches!(word.to_ascii_lowercase().as_str(), "in" | "for" | "using"))
            .collect::<Vec<_>>()
            .join(" ");

        let mut scored: Vec<(SearchTarget, i64)> = Self::targets()
            .into_iter()
            .filter_map(|target| {
                target_score(&target.name, &target_query).map(|score| (target, score))
            })
            .collect();
        scored.sort_by(|(left, left_score), (right, right_score)| {
            right_score
                .cmp(left_score)
                .then_with(|| left.name.cmp(&right.name))
        });
        Some(
            scored
                .into_iter()
                .take(12)
                .map(|(target, _)| target)
                .collect(),
        )
    }
}

impl SearchProvider for MetaSearcher {
    fn name(&self) -> String {
        "search".to_string()
    }

    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let Some(targets) = Self::search_targets(query.trim()) else {
            return SearchResult::list(Vec::new());
        };

        let results = targets
            .into_iter()
            .map(|target| {
                ResultItem::new(
                    format!("Search {}", target.name),
                    vec![Action::new(
                        "Switch Searcher",
                        ActionData::SetQuery {
                            query: target.trigger,
                        },
                    )],
                )
                .description(format!("Continue in {} search", target.name))
            })
            .collect();

        SearchResult::list(results)
    }
}

fn trigger_query(pattern: &str) -> Option<String> {
    regex::Regex::new(pattern).ok()?;
    let body = pattern.strip_prefix('^').unwrap_or(pattern);
    let literal: String = body
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '!' | '=' | '#' | '+' | '-')
        })
        .collect();

    if literal.is_empty() {
        return None;
    }

    let rest = &body[literal.len()..];
    let needs_argument = !rest.starts_with('$');
    Some(if needs_argument {
        format!("{literal} ")
    } else {
        literal
    })
}

fn target_score(name: &str, query: &str) -> Option<i64> {
    if query.trim().is_empty() {
        return Some(0);
    }

    let normalized_name = crate::search_utils::normalize_text(name);
    let name_words: Vec<&str> = normalized_name.split_whitespace().collect();
    let mut total = 0;

    for token in crate::search_utils::normalize_text(query).split_whitespace() {
        let score = name_words
            .iter()
            .map(|word| {
                if *word == token {
                    1000
                } else if word.starts_with(&token) {
                    850
                } else if word.contains(&token) {
                    700
                } else if token.len() >= 3 {
                    crate::search_utils::smart_match(word, &token)
                        .unwrap_or(0)
                        .min(500)
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);

        if score == 0 {
            return None;
        }
        total += score;
    }

    Some(total)
}
