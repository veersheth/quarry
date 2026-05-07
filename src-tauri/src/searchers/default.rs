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
    settings::SettingsSearcher,
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

pub struct DefaultSearcher;

impl DefaultSearcher {
    pub fn new() -> Self {
        Self
    }

    fn score_item(item: &ResultItem, query: &str) -> i64 {
        let q = query.to_lowercase();
        let name = item.name.to_lowercase();
        let desc = item.description.as_deref().unwrap_or("").to_lowercase();
        let combined = format!("{} {}", name, desc);
        let combined_score = MATCHER.fuzzy_match(&combined, &q).unwrap_or(0);
        let name_score = MATCHER.fuzzy_match(&name, &q)
            .map(|s| s * 2)
            .unwrap_or(0);
        combined_score.max(name_score)
    }

    fn score_and_filter(items: Vec<ResultItem>, q: &str) -> Vec<(ResultItem, i64)> {
        items.into_iter().filter_map(|item| {
            let score = Self::score_item(&item, q);
            if score > 0 { Some((item, score)) } else { None }
        }).collect()
    }
}

impl SearchProvider for DefaultSearcher {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let q = query.trim();

        if q.is_empty() {
            return SearchResult {
                results: AppSearcher.search(q, app).results,
                result_type: ResultType::Home,
                            ..Default::default()
};
        }

        let app_clone = app.clone();
        let q_owned = q.to_string();

        // Snapshot the current sequence so we can abort if a newer query arrives.
        let my_seq = crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed);

        // Kick off the file search immediately in a rayon thread so it runs
        // concurrently while we compute and emit the fast in-memory results.
        let (file_tx, file_rx) = mpsc::channel::<Vec<ResultItem>>();
        let q_for_file = q_owned.clone();
        rayon::spawn(move || {
            // Don't bother if a newer search is already queued.
            if crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed) != my_seq {
                let _ = file_tx.send(vec![]);
                return;
            }
            let results = FileSearcher.search(&q_for_file, &app_clone).results;
            let _ = file_tx.send(results);
        });

        // Fast in-memory searchers — run in parallel, finish in ~2ms.
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

        // Build and emit partial results so the frontend can show something
        // immediately while the file search is still running.
        {
            let mut fast_scored: Vec<(ResultItem, i64)> = Vec::new();
            fast_scored.extend(Self::score_and_filter(app_results.clone(), q));
            fast_scored.extend(Self::score_and_filter(sys_results.clone(), q));
            fast_scored.extend(Self::score_and_filter(bookmark_results.clone(), q));
            fast_scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));

            let mut fast_seen: HashSet<String> = HashSet::new();
            let mut fast_shortcuts: Vec<ResultItem> = shortcut_results.iter()
                .filter(|i| {
                    Self::score_item(i, q) > 0 && fast_seen.insert(i.name.to_lowercase())
                })
                .take(3)
                .cloned()
                .collect();

            let fast_rest: Vec<ResultItem> = fast_scored.into_iter()
                .filter_map(|(item, _)| {
                    if fast_seen.insert(item.name.to_lowercase()) { Some(item) } else { None }
                })
                .collect();

            fast_shortcuts.extend(fast_rest);

            let _ = app.emit("quarry-fast", FastPartial {
                query: q.to_string(),
                results: fast_shortcuts,
            });
        }

        // If a newer query already arrived while we were computing fast results,
        // bail out now — the command handler will discard this result anyway,
        // and blocking on file_rx would only waste rayon threads.
        if crate::SEARCH_SEQ.load(std::sync::atomic::Ordering::Relaxed) != my_seq {
            return SearchResult { results: vec![], result_type: ResultType::List, ..Default::default() };
        }

        // Wait for file results (likely already done or nearly done).
        let file_results = file_rx.recv().unwrap_or_default();

        // Settings (only for longer queries).
        let settings_results = if q.len() >= 3 {
            SettingsSearcher.search(&q_owned, app).results
        } else {
            vec![]
        };

        // Score and combine all results.
        let mut scored: Vec<(ResultItem, i64)> = Vec::new();
        for item in app_results {
            let score = Self::score_item(&item, q);
            if score > 0 { scored.push((item, score)); }
        }
        for item in sys_results {
            let score = Self::score_item(&item, q);
            if score > 0 { scored.push((item, score)); }
        }
        for item in file_results {
            let score = Self::score_item(&item, q);
            if score > 0 { scored.push((item, score)); }
        }
        for item in bookmark_results {
            let score = Self::score_item(&item, q);
            if score > 0 { scored.push((item, score)); }
        }

        let mut settings_scored: Vec<(ResultItem, i64)> = settings_results
            .into_iter()
            .filter_map(|item| {
                let score = Self::score_item(&item, q);
                if score > 0 { Some((item, score)) } else { None }
            })
            .collect();
        settings_scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        settings_scored.truncate(3);
        scored.extend(settings_scored);

        scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let mut seen_names: HashSet<String> = HashSet::new();

        let mut top_shortcuts: Vec<ResultItem> = shortcut_results
            .into_iter()
            .filter(|item| {
                let score = Self::score_item(item, q);
                score > 0 && seen_names.insert(item.name.to_lowercase())
            })
            .take(3)
            .collect();

        let mut combined: Vec<ResultItem> = scored
            .into_iter()
            .filter_map(|(item, _)| {
                let key = item.name.to_lowercase();
                if seen_names.insert(key) { Some(item) } else { None }
            })
            .collect();

        let mut emojis = EmojiSearcher.search(q, app).results;
        emojis.truncate(6);
        combined.extend(emojis);

        let mut res = MathSearcher.search(q, app).results;
        res.truncate(2);
        combined.extend(res);

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
                        name:         ws.name.clone(),
                        url_template: ws.url.clone(),
                        icon:         ws.icon.clone(),
                    };
                    let mut results = searcher.search(q, app).results;
                    results.truncate(max);
                    combined.extend(results);
                }
            }
        }

        top_shortcuts.extend(combined);
        SearchResult {
            results: top_shortcuts,
            result_type: ResultType::List,
                    ..Default::default()
}
    }
}
