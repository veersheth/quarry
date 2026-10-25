use tauri::AppHandle;

use crate::types::{Action, ActionData, ResultItem, SearchResult};
use crate::CONFIG;

use super::SearchProvider;

#[derive(Clone)]
struct SearchTarget {
    name: String,
    config_key: String,
    trigger: String,
    regex: String,
    pattern: regex::Regex,
}

pub struct MetaSearcher;

impl MetaSearcher {
    fn targets() -> Vec<SearchTarget> {
        let Ok(config) = CONFIG.read() else {
            return Vec::new();
        };

        let triggers = &config.triggers;
        let configured = [
            ("Apps", "apps", triggers.apps.as_str()),
            ("Files", "files", triggers.files.as_str()),
            ("Clipboard", "clipboard", triggers.clipboard.as_str()),
            ("Emojis", "emojis", triggers.emojis.as_str()),
            ("Fonts", "fonts", triggers.fonts.as_str()),
            ("Bookmarks", "bookmarks", triggers.bookmarks.as_str()),
            ("System", "system", triggers.system.as_str()),
            ("Shortcuts", "shortcuts", triggers.shortcuts.as_str()),
            ("Scripts", "scripts", triggers.scripts.as_str()),
            ("Screenshots", "screenshots", triggers.screenshots.as_str()),
            ("Math", "math", triggers.math.as_str()),
            ("Dictionary", "dictionary", triggers.dictionary.as_str()),
            ("Time", "time", triggers.time.as_str()),
            ("Timer", "timer", triggers.timer.as_str()),
            ("AI", "ai", triggers.ai.as_str()),
            ("Camera", "camera", triggers.camera.as_str()),
            ("Color Picker", "color_picker", triggers.color_picker.as_str()),
            ("Currency", "currency", triggers.currency.as_str()),
            ("QR Code", "qr_code", triggers.qr_code.as_str()),
        ];

        let mut targets: Vec<SearchTarget> = configured
            .into_iter()
            .filter_map(|(name, config_key, pattern)| {
                let trigger = trigger_query(pattern)?;
                let pattern = regex::Regex::new(pattern).ok()?;
                Some(SearchTarget {
                    name: name.to_string(),
                    config_key: config_key.to_string(),
                    trigger,
                    regex: pattern.as_str().to_string(),
                    pattern,
                })
            })
            .collect();

        targets.extend(config.web_searches.iter().filter_map(|web_search| {
            let trigger = trigger_query(&web_search.trigger)?;
            let pattern = regex::Regex::new(&web_search.trigger).ok()?;
            Some(SearchTarget {
                name: format!("{} Search", web_search.name),
                config_key: format!("web:{}", web_search.name),
                trigger,
                regex: web_search.trigger.clone(),
                pattern,
            })
        }));
        targets
    }

    /// Return shortcut items for the most recently used configured searchers.
    pub fn recent_searchers(limit: usize) -> Vec<ResultItem> {
        let targets = Self::targets();
        let mut recent: Vec<(u64, SearchTarget)> = Vec::new();

        for entry in crate::usage_tracker::get_recent_entries("", 1_000) {
            let target = if let Some(trigger) = entry.action_id.strip_prefix("searcher:") {
                targets
                    .iter()
                    .find(|target| target.trigger.eq_ignore_ascii_case(trigger))
            } else {
                targets
                    .iter()
                    .find(|target| target.pattern.is_match(&entry.query))
            };

            if let Some(target) = target {
                if !recent
                    .iter()
                    .any(|(_, existing)| existing.trigger.eq_ignore_ascii_case(&target.trigger))
                {
                    recent.push((entry.last_used, target.clone()));
                }
            }
        }

        recent.sort_by(|left, right| right.0.cmp(&left.0));
        recent
            .into_iter()
            .take(limit)
            .map(|(_, target)| Self::target_to_result(target).group("recent-searcher"))
            .collect()
    }

    fn search_targets(query: &str) -> Option<Vec<SearchTarget>> {
        let words: Vec<&str> = query.split_whitespace().collect();
        if !words
            .first()
            .is_some_and(|word| word.eq_ignore_ascii_case("search"))
        {
            return None;
        }

        let mut target_query = words
            .iter()
            .skip(1)
            .copied()
            .skip_while(|word| matches!(word.to_ascii_lowercase().as_str(), "in" | "for" | "using"))
            .collect::<Vec<_>>()
            .join(" ");
        if matches!(
            target_query.to_ascii_lowercase().as_str(),
            "searcher" | "searchers"
        ) {
            target_query.clear();
        }

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
        Some(scored.into_iter().map(|(target, _)| target).collect())
    }

    pub fn suggestions(query: &str) -> Vec<ResultItem> {
        let query = query.trim();
        let meta_query = if matches!(
            query.to_ascii_lowercase().as_str(),
            "searcher" | "searchers"
        ) {
            "search".to_string()
        } else {
            format!("search {query}")
        };
        Self::search_targets(&meta_query)
            .unwrap_or_default()
            .into_iter()
            .map(Self::target_to_result)
            .collect()
    }

    fn target_to_result(target: SearchTarget) -> ResultItem {
        ResultItem::new(
            format!("Search {}", target.name),
            vec![
                Action::new(
                    "Switch to searcher",
                    ActionData::SetQuery {
                        query: target.trigger.clone(),
                    },
                ),
                Action::new(
                    "Set prefix",
                    ActionData::RunFunction {
                        function_name: "edit_searcher_trigger".into(),
                        params: vec![
                            target.config_key.clone(),
                            target.name.clone(),
                            "prefix".into(),
                            target.trigger.trim().to_string(),
                        ],
                    },
                ),
                Action::new(
                    "Set trigger regex pattern",
                    ActionData::RunFunction {
                        function_name: "edit_searcher_trigger".into(),
                        params: vec![
                            target.config_key,
                            target.name.clone(),
                            "regex".into(),
                            target.regex,
                        ],
                    },
                ),
            ],
        )
        .description(format!("Continue in {} search", target.name))
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

        let results = targets.into_iter().map(Self::target_to_result).collect();

        SearchResult::list(results)
    }
}

fn trigger_query(pattern: &str) -> Option<String> {
    regex::Regex::new(pattern).ok()?;
    let body = pattern.strip_prefix('^').unwrap_or(pattern);
    let mut literal = String::new();
    let mut consumed = 0;
    while consumed < body.len() {
        let character = body[consumed..].chars().next().unwrap();
        if !character.is_whitespace()
            && character != '\\'
            && !matches!(character, '.' | '+' | '*' | '?' | '[' | ']' | '^' | '$' | '(' | ')' | '{' | '}' | '|')
        {
            literal.push(character);
            consumed += character.len_utf8();
        } else if character == '\\' {
            let Some(escaped) = body[consumed + 1..].chars().next() else {
                break;
            };
            if escaped.is_alphanumeric() || escaped.is_whitespace() {
                break;
            }
            literal.push(escaped);
            consumed += 1 + escaped.len_utf8();
        } else {
            break;
        }
    }

    if literal.is_empty() {
        return None;
    }

    let rest = &body[consumed..];
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
