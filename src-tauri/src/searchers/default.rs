use tauri::AppHandle;
use once_cell::sync::Lazy;
use regex::Regex;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashSet;
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
    system::SystemSearcher,
    web_searchers::WebSearcher,
    SearchProvider,
};

use crate::types::{ResultItem, ResultType, SearchResult};

// Matches: "100 USD EUR", "100 USD to EUR", "USD EUR", "USD to EUR"
static CURRENCY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(\d[\d,.]*)?\s*([a-z]{3})\s+(?:to\s+)?([a-z]{3})$").unwrap()
});

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

pub struct DefaultSearcher;

impl DefaultSearcher {
    pub fn new() -> Self {
        Self
    }

    fn score_item(item: &ResultItem, query: &str) -> i64 {
        let combined = format!(
            "{} {}",
            item.name,
            item.description.as_deref().unwrap_or("")
        );
        let combined_score = MATCHER.fuzzy_match(&combined, query).unwrap_or(0);
        let name_score = MATCHER.fuzzy_match(&item.name, query)
            .map(|s| s * 2)
            .unwrap_or(0);
        combined_score.max(name_score)
    }
}

impl SearchProvider for DefaultSearcher {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let q = query.trim();

        if q.is_empty() {
            return SearchResult {
                results: AppSearcher.search(q, app).results,
                result_type: ResultType::Home,
            };
        }

        let app_handle = app.clone();
        let q_owned = q.to_string();

        let ((app_results, sys_results), (file_results, bookmark_results)) = rayon::join(
            || rayon::join(
                || AppSearcher.search(&q_owned, &app_handle).results,
                || SystemSearcher.search(&q_owned, &app_handle).results,
            ),
            || rayon::join(
                || FileSearcher.search(&q_owned, &app_handle).results,
                || BookmarksSearcher.search(&q_owned, &app_handle).results,
            ),
        );

        let settings_results = if q.len() >= 3 {
            SettingsSearcher.search(&q_owned, &app_handle).results
        } else {
            vec![]
        };

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

        // Bare currency query: "100 INR USD", "100 inr to usd", "gbp eur", etc.
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

        SearchResult {
            results: combined,
            result_type: ResultType::List,
        }
    }
}
