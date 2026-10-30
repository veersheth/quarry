use tauri::AppHandle;

use crate::types::{Action, ActionData, ResultItem, SearchResult};
use crate::CONFIG;

use super::SearchProvider;

#[derive(Clone)]
struct SearchTarget {
    badge: String,
    command: String,
    description: String,
    search_terms: Vec<String>,
    config_key: String,
    trigger: String,
    regex: String,
    pattern: regex::Regex,
}

pub struct MetaSearcher;

impl MetaSearcher {
    /// Human-facing labels for searchers. Searcher IDs and configured regexes stay stable.
    fn presentation(
        config_key: &str,
    ) -> (
        &'static str,
        &'static str,
        &'static str,
        &'static [&'static str],
    ) {
        match config_key {
            "apps" => (
                "Apps",
                "Search Apps",
                "Find and launch apps",
                &["application", "launch"],
            ),
            "files" => (
                "Files",
                "Search Files",
                "Find files and folders",
                &["folder", "document"],
            ),
            "clipboard" => (
                "Clipboard",
                "Search Clipboard",
                "Find copied text and images",
                &["paste", "history"],
            ),
            "emojis" => (
                "Emojis",
                "Search Emojis",
                "Find and paste emoji",
                &["emoji", "symbols"],
            ),
            "fonts" => (
                "Fonts",
                "Search Fonts",
                "Preview and copy fonts",
                &["typeface"],
            ),
            "bookmarks" => (
                "Bookmarks",
                "Search Bookmarks",
                "Find saved links",
                &["bookmark", "url"],
            ),
            "system" => (
                "System",
                "Open System Actions",
                "Control system and session settings",
                &["power", "lock", "sleep"],
            ),
            "shortcuts" => (
                "Shortcuts",
                "Search Shortcuts",
                "Find built-in Quarry actions",
                &["action", "command"],
            ),
            "scripts" => (
                "Scripts",
                "Search Scripts",
                "Find and run saved scripts",
                &["script"],
            ),
            "screenshots" => (
                "Screenshots",
                "Browse Screenshots",
                "Find screenshots and extract text",
                &["screenshot", "ocr"],
            ),
            "math" => (
                "Calculator",
                "Calculate",
                "Evaluate a math expression",
                &["math", "calculator"],
            ),
            "dictionary" => (
                "Dictionary",
                "Define a Word",
                "Look up a definition",
                &["definition", "word"],
            ),
            "time" => (
                "World Clock",
                "Check World Time",
                "Look up time in a city or time zone",
                &["time", "clock", "timezone"],
            ),
            "timer" => ("Timer", "Set a Timer", "Start a countdown", &["countdown"]),
            "ai" => ("AI", "Ask AI", "Ask a question", &["assistant", "chat"]),
            "camera" => (
                "Camera",
                "Preview Camera",
                "Open a live camera preview",
                &["webcam", "preview"],
            ),
            "color_picker" => (
                "Color Picker",
                "Pick a Color",
                "Choose a color from the screen",
                &["color", "colour"],
            ),
            "currency" => (
                "Currency",
                "Convert Currency",
                "Convert between currencies",
                &["exchange", "money"],
            ),
            "qr_code" => (
                "QR Code",
                "Create QR Code",
                "Make a QR code from text or a link",
                &["qr", "barcode"],
            ),
            "shell" => (
                "Shell",
                "Run a Shell Command",
                "Execute a terminal command",
                &["terminal", "command"],
            ),
            "lorem" => (
                "Lorem Ipsum",
                "Generate Placeholder Text",
                "Create lorem ipsum text",
                &["lorem", "placeholder"],
            ),
            _ => ("", "", "", &[]),
        }
    }

    pub fn badge_for_provider(provider: &str) -> String {
        let config_key = match provider {
            "color" => "color_picker",
            "qr" => "qr_code",
            _ => provider,
        };
        let (badge, _, _, _) = Self::presentation(config_key);
        if !badge.is_empty() {
            return badge.to_string();
        }
        if provider == "search" {
            return "Searchers".to_string();
        }
        provider
            .strip_prefix("web:")
            .unwrap_or(provider)
            .to_string()
    }

    fn targets() -> Vec<SearchTarget> {
        let Ok(config) = CONFIG.read() else {
            return Vec::new();
        };

        let triggers = &config.triggers;
        let configured = [
            ("apps", triggers.apps.as_str()),
            ("files", triggers.files.as_str()),
            ("clipboard", triggers.clipboard.as_str()),
            ("emojis", triggers.emojis.as_str()),
            ("fonts", triggers.fonts.as_str()),
            ("bookmarks", triggers.bookmarks.as_str()),
            ("system", triggers.system.as_str()),
            ("shortcuts", triggers.shortcuts.as_str()),
            ("scripts", triggers.scripts.as_str()),
            ("screenshots", triggers.screenshots.as_str()),
            ("math", triggers.math.as_str()),
            ("dictionary", triggers.dictionary.as_str()),
            ("time", triggers.time.as_str()),
            ("timer", triggers.timer.as_str()),
            ("ai", triggers.ai.as_str()),
            ("camera", triggers.camera.as_str()),
            ("color_picker", triggers.color_picker.as_str()),
            ("currency", triggers.currency.as_str()),
            ("qr_code", triggers.qr_code.as_str()),
            ("shell", triggers.shell.as_str()),
            ("lorem", triggers.lorem.as_str()),
        ];

        let mut targets: Vec<SearchTarget> = configured
            .into_iter()
            .filter_map(|(config_key, pattern)| {
                let trigger = trigger_query(pattern)?;
                let pattern = regex::Regex::new(pattern).ok()?;
                let (badge, command, description, terms) = Self::presentation(config_key);
                Some(SearchTarget {
                    badge: badge.to_string(),
                    command: command.to_string(),
                    description: description.to_string(),
                    search_terms: terms.iter().map(|term| (*term).to_string()).collect(),
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
                badge: web_search.name.clone(),
                command: format!("Search {}", web_search.name),
                description: format!("Search the web with {}", web_search.name),
                search_terms: vec!["web".into(), "internet".into()],
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
            .filter_map(|target| target_score(&target, &target_query).map(|score| (target, score)))
            .collect();
        scored.sort_by(|(left, left_score), (right, right_score)| {
            right_score
                .cmp(left_score)
                .then_with(|| left.command.cmp(&right.command))
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
        let mut actions = vec![Action::new(
            "Switch to searcher",
            ActionData::SetQuery {
                query: target.trigger.clone(),
            },
        )];

        if !target.trigger.trim().is_empty() {
            actions.push(Action::new(
                "Set prefix",
                ActionData::RunFunction {
                    function_name: "edit_searcher_trigger".into(),
                    params: vec![
                        target.config_key.clone(),
                        target.command.clone(),
                        "prefix".into(),
                        target.trigger.trim().to_string(),
                    ],
                },
            ));
        }

        actions.push(Action::new(
            "Set trigger regex pattern",
            ActionData::RunFunction {
                function_name: "edit_searcher_trigger".into(),
                params: vec![
                    target.config_key,
                    target.command.clone(),
                    "regex".into(),
                    target.regex,
                ],
            },
        ));

        ResultItem::new(target.command, actions).description(target.description)
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
            && !matches!(
                character,
                '.' | '+' | '*' | '?' | '[' | ']' | '^' | '$' | '(' | ')' | '{' | '}' | '|'
            )
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

    if literal.is_empty() && body.starts_with(' ') {
        return Some(" ".to_string());
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

fn target_score(target: &SearchTarget, query: &str) -> Option<i64> {
    if query.trim().is_empty() {
        return Some(0);
    }

    let searchable = std::iter::once(target.command.as_str())
        .chain(std::iter::once(target.badge.as_str()))
        .chain(target.search_terms.iter().map(String::as_str))
        .collect::<Vec<_>>();
    crate::search_utils::match_all_tokens(&searchable, query)
}
