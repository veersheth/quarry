use tauri::{AppHandle, Emitter};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashSet;
use std::sync::mpsc;
use rayon;

use crate::searchers::{
    apps::AppSearcher,
    currency::CurrencySearcher,
    emojis::EmojiSearcher,
    files::FileSearcher,
    bookmarks::BookmarksSearcher,
    math::MathSearcher,
    shell::ShellSearcher,
    shortcuts::ShortcutsSearcher,
    system::SystemSearcher,
    web_searchers::WebSearcher,
    SearchProvider,
};

use crate::types::{ResultItem, ResultType, SearchResult};

static CURRENCY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(\d[\d,.]*)?\s*([a-z]{3})\s+(?:to\s+)?([a-z]{3})$").unwrap()
});

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

#[derive(Serialize, Clone)]
struct FastPartial {
    query: String,
    results: Vec<ResultItem>,
}

/// Source type for scoring - apps/shortcuts always beat files at equal fuzzy quality.
#[derive(Clone, Copy)]
enum Source {
    App,
    Shortcut,
    System,
    Bookmark,
    File,
}

pub struct DefaultSearcher;

impl DefaultSearcher {
    pub fn new() -> Self { Self }

    /// Score an item against a query, taking source type into account.
    ///
    /// Score = raw_skim × match_quality_multiplier × type_multiplier
    ///
    /// match_quality tiers (for name field):
    ///   exact match        → 4.0×
    ///   prefix match       → 2.5×  ("ghost" → "ghostty")
    ///   word-start match   → 1.8×  ("chr" → "Google Chrome", "chrome" word starts with "chr")
    ///   substring match    → 1.3×  ("file" somewhere in name)
    ///   fuzzy-only         → 1.0×
    ///
    /// type multipliers:
    ///   App 2.0, Shortcut 1.7, System 1.2, Bookmark 1.0, File 0.75
    ///
    /// This means "ghostty" (app, prefix match) will always beat "ghosts-yummy.txt"
    /// (file, also prefix match) because 2.0 >> 0.75, and skim scores shorter/denser
    /// matches higher anyway. Files only beat apps when their fuzzy score is dramatically
    /// higher and the app has only a weak match.
    fn score_item(item: &ResultItem, query: &str, source: Source) -> i64 {
        let q = query.to_lowercase();
        let name = item.name.to_lowercase();
        let desc = item.description.as_deref().unwrap_or("").to_lowercase();

        // Also try normalized matching for better results
        let norm_name = crate::search_utils::normalize_text(&item.name);
        let norm_query = crate::search_utils::normalize_text(query);

        let raw_name = MATCHER.fuzzy_match(&name, &q).unwrap_or(0);
        let raw_desc = MATCHER.fuzzy_match(&desc, &q).unwrap_or(0);
        let raw_norm = MATCHER.fuzzy_match(&norm_name, &norm_query).unwrap_or(0);

        if raw_name == 0 && raw_desc == 0 && raw_norm == 0 {
            return 0;
        }

        // Name match quality - determines how "intentional" the match looks
        let name_quality: f64 = if !q.is_empty() && raw_name > 0 {
            if name == q {
                4.0
            } else if name.starts_with(&q) {
                2.5
            } else if name.split_whitespace().any(|w| w.starts_with(&q)) {
                // word-start: "chr" matches the "Chrome" word in "Google Chrome"
                1.8
            } else if name.contains(&q) {
                1.3
            } else {
                1.0 // scattered fuzzy
            }
        } else if !norm_query.is_empty() && raw_norm > 0 {
            // Check normalized matches too
            if norm_name == norm_query {
                3.5 // slightly lower than exact original match
            } else if norm_name.starts_with(&norm_query) {
                2.2
            } else if norm_name.split_whitespace().any(|w| w.starts_with(&norm_query)) {
                1.6
            } else if norm_name.contains(&norm_query) {
                1.1
            } else {
                0.8 // normalized fuzzy gets lower priority
            }
        } else {
            1.0
        };

        // Name counts double; desc is a tiebreaker only
        // Use the best score from original or normalized matching
        let best_name_score = raw_name.max(raw_norm);
        let base = if best_name_score > 0 {
            (best_name_score as f64 * name_quality * 2.0) as i64
        } else {
            raw_desc
        };

        let type_mult: f64 = match source {
            Source::App      => 2.0,
            Source::Shortcut => 1.7,
            Source::System   => 1.2,
            Source::Bookmark => 1.0,
            Source::File     => 0.75,
        };

        (base as f64 * type_mult) as i64
    }

    fn score_items(items: Vec<ResultItem>, query: &str, source: Source) -> Vec<(ResultItem, i64)> {
        items.into_iter().filter_map(|item| {
            let s = Self::score_item(&item, query, source);
            if s > 0 { Some((item, s)) } else { None }
        }).collect()
    }

    /// Merge multiple scored lists: sort by score descending, deduplicate by name.
    fn merge_scored(groups: Vec<Vec<(ResultItem, i64)>>, seen: &mut HashSet<String>) -> Vec<ResultItem> {
        let mut flat: Vec<(ResultItem, i64)> = groups.into_iter().flatten().collect();
        flat.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        flat.into_iter()
            .filter_map(|(item, _)| {
                if seen.insert(item.name.to_lowercase()) { Some(item) } else { None }
            })
            .collect()
    }
}

// ── Home-screen helpers ──────────────────────────────────────────────────────

fn make_header(label: &str) -> ResultItem {
    ResultItem::new(label, vec![]).group("header")
}

/// A single calc history entry as a result item.
fn calc_result_item(expr: &str, result: &str, raw: &str) -> ResultItem {
    ResultItem::new(
        format!("{} = {}", expr, result),
        vec![crate::types::Action::new("Copy", crate::types::ActionData::CopyToClipboard { text: raw.to_string() })],
    )
    .description("Calculation")
    .icon("icons/math.png")
}

/// Build the unified "Recent" feed: interleave recently used apps and recent
/// calculations, sorted purely by timestamp (most recent first), capped at `limit`.
///
/// Returns `(recent_items, seen_names)` where `seen_names` is the lowercase set
/// of all app names already included (for deduplicating the Apps section).
fn unified_recent(
    all_apps: &[ResultItem],
    limit: usize,
) -> (Vec<ResultItem>, HashSet<String>) {
    // --- App usage: sorted by last_used descending --------------------------
    let mut usage = crate::usage_tracker::get_recent_entries("", 20);
    usage.sort_by(|a, b| b.last_used.cmp(&a.last_used));

    // Map app name (lowercase) → ResultItem for O(1) lookup
    let app_by_name: std::collections::HashMap<String, &ResultItem> = all_apps
        .iter()
        .map(|a| (a.name.to_lowercase(), a))
        .collect();

    // --- Calc history: up to 3 recent calcs --------------------------------
    let calcs = crate::searchers::math::recent_calcs(3);

    // --- Merge both streams into (timestamp, ResultItem) pairs -------------
    // Apps: pull from usage entries, resolve to actual ResultItem
    let mut seen_apps: HashSet<String> = HashSet::new();
    let mut events: Vec<(u64, ResultItem)> = Vec::new();

    for entry in &usage {
        let key = entry.name.to_lowercase();
        if seen_apps.contains(&key) { continue; }
        if let Some(&app_item) = app_by_name.get(&key) {
            seen_apps.insert(key);
            events.push((entry.last_used, app_item.clone()));
        }
    }

    // Calcs
    for (expr, result, raw, ts) in &calcs {
        events.push((*ts, calc_result_item(expr, result, raw)));
    }

    // Sort by timestamp descending, take `limit`
    events.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    events.truncate(limit);

    // Collect app names that made it in (for dedup against Apps section)
    let mut included_app_names: HashSet<String> = HashSet::new();
    for (_, item) in &events {
        if item.description.as_deref() != Some("calculation") {
            included_app_names.insert(item.name.to_lowercase());
        }
    }

    let items = events.into_iter().map(|(_, item)| item).collect();
    (items, included_app_names)
}

// ─────────────────────────────────────────────────────────────────────────────

impl SearchProvider for DefaultSearcher {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let q = query.trim();

        if q.is_empty() {
            let mut results = crate::searchers::home::home_items(app);

            let all_apps = AppSearcher.search("", app).results;

            // Unified recent feed (apps + calcs, sorted by time)
            let (recent_items, recent_names) = unified_recent(&all_apps, 8);

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
        let q_owned = q.to_string();
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
            || rayon::join(
                || AppSearcher.search(&q_owned, app).results,
                || SystemSearcher.search(&q_owned, app).results,
            ),
            || rayon::join(
                || BookmarksSearcher.search(&q_owned, app).results,
                || ShortcutsSearcher.search(&q_owned, app).results,
            ),
        );

        // Emit fast partial results with the same scoring as the final pass
        {
            let mut seen = HashSet::new();
            let fast = Self::merge_scored(vec![
                Self::score_items(app_results.clone(), q, Source::App),
                Self::score_items(shortcut_results.clone(), q, Source::Shortcut),
                Self::score_items(sys_results.clone(), q, Source::System),
                Self::score_items(bookmark_results.clone(), q, Source::Bookmark),
            ], &mut seen);
            let _ = app.emit("quarry-fast", FastPartial { query: q.to_string(), results: fast });
        }

        if crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed) != my_seq {
            return SearchResult { results: vec![], result_type: ResultType::List, ..Default::default() };
        }

        let file_results = file_rx.recv().unwrap_or_default();

        // Merge all primary sources - score determines order, no hard-coded pinning
        let mut seen = HashSet::new();
        let mut combined = Self::merge_scored(vec![
            Self::score_items(app_results, q, Source::App),
            Self::score_items(shortcut_results, q, Source::Shortcut),
            Self::score_items(sys_results, q, Source::System),
            Self::score_items(bookmark_results, q, Source::Bookmark),
            Self::score_items(file_results, q, Source::File),
        ], &mut seen);

        // Supplementary results - appended after ranked items, not competing by score
        let mut emojis = EmojiSearcher.search(q, app).results;
        emojis.truncate(6);
        combined.extend(emojis);

        let mut math = MathSearcher.search(q, app).results;
        math.truncate(2);
        combined.extend(math);

        // Currency result is so specific that if it matches we surface it first
        if CURRENCY_RE.is_match(q) {
            let mut res = CurrencySearcher.search(q, app).results;
            res.truncate(1);
            let tail = combined;
            combined = res;
            combined.extend(tail);
        }

        if q.len() >= 3 {
            let mut sh = ShellSearcher.search(q, app).results;
            sh.truncate(2);
            combined.extend(sh);
        }

        if q.len() >= 2 {
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
                    let mut results = searcher.search(q, app).results;
                    results.truncate(max);
                    combined.extend(results);
                }
            }
        }

        SearchResult {
            results: combined,
            result_type: ResultType::List,
            ..Default::default()
        }
    }
}
