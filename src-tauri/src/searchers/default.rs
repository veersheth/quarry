use once_cell::sync::Lazy;
use rayon;
use regex::Regex;
use std::collections::HashSet;
use std::sync::mpsc;
use tauri::AppHandle;

use crate::searchers::{
    apps::AppSearcher, bookmarks::BookmarksSearcher, currency::CurrencySearcher,
    emojis::EmojiSearcher, files::FileSearcher, math::MathSearcher, meta::MetaSearcher,
    scripts::ScriptsSearcher, shortcuts::ShortcutsSearcher, system::SystemSearcher,
    web_searchers::WebSearcher, SearchProvider,
};
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

static CURRENCY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(\d[\d,.]*)?\s*([a-z]{3})\s+(?:to\s+)?([a-z]{3})$").unwrap());

/// Source type for scoring - apps/shortcuts always beat files at equal fuzzy quality.
#[derive(Clone, Copy)]
enum Source {
    App,
    Shortcut,
    System,
    Bookmark,
    Script,
    File,
}

impl Source {
    fn tie_breaker(self) -> i64 {
        match self {
            Self::App => 50,
            Self::Shortcut => 40,
            Self::System => 30,
            Self::Bookmark => 20,
            Self::Script => 10,
            Self::File => 0,
        }
    }
}

fn field_score(field: &str, token: &str) -> i64 {
    crate::search_utils::token_match_score(field, token)
}

struct SearchTerms {
    source_query: String,
    normalized: String,
    tokens: Vec<String>,
}

impl SearchTerms {
    fn new(query: &str) -> Self {
        let source_query = strip_intent_prefix(query);
        let source_query = if source_query.is_empty() {
            query.trim()
        } else {
            source_query
        }
        .to_string();
        let normalized = crate::search_utils::normalize_text(&source_query);
        let tokens: Vec<String> = normalized.split_whitespace().map(str::to_string).collect();
        Self {
            source_query,
            tokens,
            normalized,
        }
    }
}

fn strip_intent_prefix(query: &str) -> &str {
    let mut remainder = query.trim_start();
    loop {
        let Some((word, rest)) = remainder.split_once(char::is_whitespace) else {
            return if is_search_intent(remainder) {
                ""
            } else {
                remainder
            };
        };
        if !is_search_intent(word) {
            return remainder;
        }
        remainder = rest.trim_start();
    }
}

fn is_search_intent(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "find" | "launch" | "open" | "run" | "show" | "start" | "switch" | "to"
    )
}

pub struct DefaultSearcher;

impl DefaultSearcher {
    fn score_item(item: &ResultItem, terms: &SearchTerms, source: Source) -> i64 {
        if terms.tokens.is_empty() {
            return 0;
        }

        let name = crate::search_utils::normalize_text(&item.name);
        let description = item
            .description
            .as_deref()
            .map(crate::search_utils::normalize_text)
            .unwrap_or_default();
        let action_names = item
            .actions
            .iter()
            .map(|action| crate::search_utils::normalize_text(&action.name))
            .filter(|name| {
                !matches!(
                    name.as_str(),
                    "add" | "copy" | "delete" | "launch" | "open" | "paste" | "run" | "select"
                )
            })
            .collect::<Vec<_>>();

        let mut score = 0;
        for token in &terms.tokens {
            let name_score = field_score(&name, token);
            let description_score = field_score(&description, token) * 55 / 100;
            let action_score = action_names
                .iter()
                .map(|action| field_score(action, token) * 65 / 100)
                .max()
                .unwrap_or(0);
            let best = name_score.max(description_score).max(action_score);
            if best == 0 {
                return 0;
            }
            score += best;
        }

        if name == terms.normalized {
            score += 1_200;
        } else if name.starts_with(&terms.normalized) {
            score += 500;
        } else if name.contains(&terms.normalized) {
            score += 250;
        }

        // Source is only a small tie-breaker. A strong file-name match should
        // beat a weak app-description match.
        score + source.tie_breaker()
    }

    fn score_items(
        items: Vec<ResultItem>,
        terms: &SearchTerms,
        source: Source,
    ) -> Vec<(ResultItem, i64)> {
        items
            .into_iter()
            .filter_map(|item| {
                let s = Self::score_item(&item, terms, source);
                if s > 0 {
                    Some((item, s))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Merge multiple scored lists: sort by score descending, deduplicate by name.
    fn merge_scored(
        groups: Vec<Vec<(ResultItem, i64)>>,
        seen: &mut HashSet<String>,
    ) -> Vec<ResultItem> {
        let mut flat: Vec<(ResultItem, i64)> = groups.into_iter().flatten().collect();
        flat.sort_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| {
                crate::search_utils::normalize_text(&left.0.name)
                    .cmp(&crate::search_utils::normalize_text(&right.0.name))
            })
        });
        flat.into_iter()
            .filter_map(|(item, _)| {
                if seen.insert(item.name.to_lowercase()) {
                    Some(item)
                } else {
                    None
                }
            })
            .collect()
    }
}

// ── Home-screen helpers ──────────────────────────────────────────────────────

fn make_header(label: &str) -> ResultItem {
    ResultItem::new(label, vec![]).group("header")
}

/// Build a ResultItem for a usage entry, inferring the action and icon from the
/// stable action_id prefix. Returns None for entries that shouldn't appear on the
/// home screen (clipboard copies, rofi, internal functions).
fn item_from_usage(
    entry: &crate::usage_tracker::UsageEntry,
    app_by_name: &std::collections::HashMap<String, &ResultItem>,
    shortcut_by_id: &std::collections::HashMap<String, &ResultItem>,
) -> Option<ResultItem> {
    let id = entry.action_id.as_str();

    // Skip clipboard copies and internal/meta actions
    if id.starts_with("copy:")
        || id.starts_with("copy-img:")
        || id.starts_with("copy-file:")
        || id.starts_with("rofi:")
        || id == "none"
        || id.starts_with("fn:delete_")
        || id.starts_with("fn:clear_")
        || id.starts_with("fn:unpin")
        || id.starts_with("fn:pin")
    {
        return None;
    }

    if id.starts_with("app:") {
        let key = entry.name.to_lowercase();
        if let Some(&app) = app_by_name.get(&key) {
            return Some(app.clone());
        }
        // App was removed — show ghost entry with just the name
        let exec = id.trim_start_matches("app:");
        return Some(ResultItem::new(
            entry.name.clone(),
            vec![Action::new(
                "Launch",
                ActionData::LaunchApp {
                    executable: exec.to_string(),
                    args: vec![],
                },
            )],
        ));
    }

    if id.starts_with("script:") {
        let path = id.trim_start_matches("script:").to_string();
        return Some(
            ResultItem::new(
                entry.name.clone(),
                vec![
                    Action::new("Run", ActionData::RunScript { path: path.clone() }),
                    Action::new(
                        "Run in Terminal",
                        ActionData::RunFunction {
                            function_name: "run_in_terminal".into(),
                            params: vec![path],
                        },
                    ),
                ],
            )
            .icon(super::SCRIPT_ICON),
        );
    }

    if id.starts_with("url:") {
        // Prefer the live shortcut/bookmark item to preserve its icon and full action list
        if let Some(&live) = shortcut_by_id.get(id) {
            return Some(live.clone());
        }
        let url = id.trim_start_matches("url:").to_string();
        let icon = if url.starts_with("file://") {
            super::ICON_FILE
        } else {
            super::ICON_BOOKMARK
        };
        return Some(
            ResultItem::new(
                entry.name.clone(),
                vec![Action::new("Open", ActionData::OpenUrl { url })],
            )
            .icon(icon),
        );
    }

    // For fn: and shell: entries, only show items that match a live shortcut.
    // This prevents modal confirmation buttons (like "Yes" for Reboot) from
    // appearing in the recent feed — their stable_id is not in any shortcut list.
    if id.starts_with("fn:") || id.starts_with("shell:") {
        return shortcut_by_id.get(id).map(|&live| live.clone());
    }

    None
}

/// Build the unified "Recent" feed from all usage history, sorted by a combined
/// recency + frequency score. Returns `(items, seen_names)` where `seen_names`
/// is used to deduplicate the Apps section below.
fn unified_recent(
    all_apps: &[ResultItem],
    all_shortcuts: &[ResultItem],
    limit: usize,
) -> (Vec<ResultItem>, HashSet<String>) {
    // Pull a larger pool so deduplication and filtering don't leave us short
    let mut usage = crate::usage_tracker::get_recent_entries("", 200);

    // Combined score: recency weighted heavily, with a frequency bonus.
    // log(count) * 2 days — so something used 10× gets ~4.6 days of bonus.
    usage.sort_unstable_by(|a, b| {
        let score = |e: &crate::usage_tracker::UsageEntry| {
            let freq_bonus = (e.count as f64).ln_1p() * 2.0 * 86400.0;
            e.last_used as f64 + freq_bonus
        };
        score(b)
            .partial_cmp(&score(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let app_by_name: std::collections::HashMap<String, &ResultItem> = all_apps
        .iter()
        .map(|a| (a.name.to_lowercase(), a))
        .collect();

    // Build stable_id → live item map for ALL known shortcuts (built-in, bookmarks, system).
    // Keyed by the stable_id of the item's primary action. Used in item_from_usage to:
    //   (a) look up the live item to get its real icon, and
    //   (b) silently drop entries whose id isn't in this map (e.g., modal "Yes"/"No" buttons).
    let shortcut_by_id: std::collections::HashMap<String, &ResultItem> = all_shortcuts
        .iter()
        .filter_map(|item| item.actions.first().map(|a| (a.data.stable_id(), item)))
        .collect();

    let mut seen: HashSet<String> = HashSet::new();
    let mut items: Vec<ResultItem> = Vec::new();

    for entry in &usage {
        // Deduplicate by action_id so the same app/file/script only appears once
        if !seen.insert(entry.action_id.clone()) {
            continue;
        }
        if let Some(item) = item_from_usage(entry, &app_by_name, &shortcut_by_id) {
            items.push(item);
            if items.len() >= limit {
                break;
            }
        }
    }

    let included_names: HashSet<String> = items.iter().map(|i| i.name.to_lowercase()).collect();

    (items, included_names)
}

// ─────────────────────────────────────────────────────────────────────────────

impl SearchProvider for DefaultSearcher {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let q = query.trim();

        if q.split_whitespace()
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("search"))
        {
            return MetaSearcher.search(q, app);
        }

        if q.is_empty() {
            let mut results = crate::searchers::home::home_items(app);

            let all_apps = AppSearcher.search("", app).results;

            // Collect all known shortcut items (built-in shortcuts + bookmarks + system actions)
            // so their stable_ids can be used to look up live icons and filter modal buttons.
            let mut all_shortcuts = ShortcutsSearcher.search("", app).results;
            all_shortcuts.extend(BookmarksSearcher.search("", app).results);
            all_shortcuts.extend(SystemSearcher.search("", app).results);

            // Unified recent feed (apps + calcs, sorted by time)
            let (recent_items, recent_names) = unified_recent(&all_apps, &all_shortcuts, 8);

            if !recent_items.is_empty() {
                results.push(make_header("Recent"));
                results.extend(recent_items);
            }

            // Apps section: apps not already shown in Recent
            let other_apps: Vec<_> = all_apps
                .into_iter()
                .filter(|item| !recent_names.contains(&item.name.to_lowercase()))
                .collect();

            if !other_apps.is_empty() {
                results.push(make_header("Apps"));
                results.extend(other_apps);
            }

            return SearchResult {
                results,
                result_type: ResultType::Home,
                ..Default::default()
            };
        }

        let app_clone = app.clone();
        let terms = SearchTerms::new(q);
        let q_owned = terms.source_query.clone();
        let my_seq = crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed);

        // Kick off file search in a rayon thread immediately so it runs
        // concurrently while we compute fast in-memory results.
        let (file_tx, file_rx) = mpsc::channel::<Vec<ResultItem>>();
        let q_for_file = q_owned.clone();
        rayon::spawn(move || {
            if crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed) != my_seq {
                let _ = file_tx.send(vec![]);
                return;
            }
            let results = FileSearcher.search(&q_for_file, &app_clone).results;
            let _ = file_tx.send(results);
        });

        // Fast in-memory searchers - parallel, finish in ~2ms
        let ((app_results, sys_results), (bookmark_results, shortcut_results)) = rayon::join(
            || {
                rayon::join(
                    || AppSearcher.search(&q_owned, app).results,
                    || SystemSearcher.search(&q_owned, app).results,
                )
            },
            || {
                rayon::join(
                    || BookmarksSearcher.search(&q_owned, app).results,
                    || ShortcutsSearcher.search(&q_owned, app).results,
                )
            },
        );
        let script_results = ScriptsSearcher.search(&q_owned, app).results;

        // Keep the ranked sources until file results are ready. The command layer
        // assigns executable action IDs only after the complete result set exists.
        let mut scored_sources = vec![
            Self::score_items(app_results, &terms, Source::App),
            Self::score_items(shortcut_results, &terms, Source::Shortcut),
            Self::score_items(sys_results, &terms, Source::System),
            Self::score_items(bookmark_results, &terms, Source::Bookmark),
            Self::score_items(script_results, &terms, Source::Script),
        ];

        if crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed) != my_seq {
            return SearchResult::list(vec![]);
        }

        let file_results = file_rx.recv().unwrap_or_default();

        // Merge all primary sources - score determines order, no hard-coded pinning
        let mut seen = HashSet::new();
        scored_sources.push(Self::score_items(file_results, &terms, Source::File));
        let mut combined = Self::merge_scored(scored_sources, &mut seen);
        let has_primary_matches = !combined.is_empty();

        // Supplementary results - appended after ranked items, not competing by score
        let mut emojis = EmojiSearcher.search(&q_owned, app).results;
        emojis.truncate(6);
        combined.extend(emojis);

        let mut math = MathSearcher.search(&q_owned, app).results;
        math.truncate(2);
        combined.extend(math);

        // Currency result is so specific that if it matches we surface it first
        if CURRENCY_RE.is_match(&q_owned) {
            let mut res = CurrencySearcher.search(&q_owned, app).results;
            res.truncate(1);
            let tail = combined;
            combined = res;
            combined.extend(tail);
        }

        // Browser searches are a fallback. When Quarry found a local match,
        // keep the result list focused and let the meta-search action switch
        // explicitly to a configured web engine.
        if !has_primary_matches && q_owned.len() >= 2 {
            let cfg_guard = crate::CONFIG.read().unwrap();
            let cfg = &*cfg_guard;
            let max = cfg.default_search.max_web_results;
            for name in &cfg.default_search.web_searches {
                if let Some(ws) = cfg.web_searches.iter().find(|w| &w.name == name) {
                    let searcher = WebSearcher {
                        name: ws.name.clone(),
                        url_template: ws.url.clone(),
                        icon: ws.icon.clone(),
                    };
                    let mut results = searcher.search(&q_owned, app).results;
                    results.truncate(max);
                    combined.extend(results);
                }
            }
        }

        SearchResult::list(combined)
    }
}
