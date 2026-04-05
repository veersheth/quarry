use tauri::AppHandle;
use regex::Regex;
use once_cell::sync::Lazy;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashSet;
use rayon;

use crate::searchers::{
    apps::AppSearcher,
    emojis::EmojiSearcher,
    files::FileSearcher,
    bookmarks::BookmarksSearcher,
    math::MathSearcher,
    shell::ShellSearcher,
    system::SystemSearcher,
    web_searchers::WebSearcher,
    SearchProvider,
};
use crate::types::{ResultItem, ResultType, SearchResult};

static MATH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([0-9+\-*/^().\s]+)$").unwrap()
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

        let (app_results, (file_results, bookmark_results)) = rayon::join(
            || AppSearcher.search(&q_owned, &app_handle).results,
            || rayon::join(
                || FileSearcher.search(&q_owned, &app_handle).results,
                || BookmarksSearcher.search(&q_owned, &app_handle).results,
            ),
        );

        let mut scored: Vec<(ResultItem, i64, u8)> = Vec::new();
        for item in app_results {
            let score = Self::score_item(&item, q);
            scored.push((item, score, 0));
        }
        for item in file_results {
            let score = Self::score_item(&item, q);
            scored.push((item, score, 1));
        }
        for item in bookmark_results {
            let score = Self::score_item(&item, q);
            scored.push((item, score, 2));
        }

        scored.sort_unstable_by(|a, b| {
            b.1.cmp(&a.1).then(a.2.cmp(&b.2))
        });

        let mut seen_names: HashSet<String> = HashSet::new();
        let mut combined: Vec<ResultItem> = scored
            .into_iter()
            .filter_map(|(item, _, _)| {
                let key = item.name.to_lowercase();
                if seen_names.insert(key) { Some(item) } else { None }
            })
            .collect();

        let mut emojis = EmojiSearcher.search(q, app).results;
        emojis.truncate(6);
        combined.extend(emojis);

        if MATH_RE.is_match(q) {
            let mut res = MathSearcher.search(q, app).results;
            res.truncate(2);
            combined.extend(res);
        }

        if q.len() >= 2 {
            let mut sys = SystemSearcher.search(q, app).results;
            sys.truncate(3);
            combined.extend(sys);
        }

        if q.len() >= 3 {
            let mut sh = ShellSearcher.search(q, app).results;
            sh.truncate(2);
            combined.extend(sh);
        }

        if q.len() >= 2 {
            let cfg = crate::config::Config::load();
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
