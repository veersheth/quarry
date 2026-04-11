use super::super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::usage_tracker::get_recent_entries;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tauri::AppHandle;

pub struct URLSearcher;

fn looks_like_url(q: &str) -> bool {
    if q.starts_with("http://") || q.starts_with("https://") {
        return true;
    }
    let without_path = q.split('/').next().unwrap_or("");
    let without_port = without_path.split(':').next().unwrap_or("");
    without_port.contains('.') || without_port == "localhost"
}

fn normalise_url(q: &str) -> String {
    if q.starts_with("http://") || q.starts_with("https://") {
        q.to_string()
    } else {
        format!("https://{}", q)
    }
}

fn term_from_name(name: &str) -> &str {
    name.strip_prefix("Open '")
        .and_then(|s| s.strip_suffix('\''))
        .unwrap_or(name)
}

fn make_item(label: String, url: String, description: impl Into<String>) -> ResultItem {
    ResultItem::new(label, ActionData::OpenUrl { url })
        .description(description)
        .icon("")
}

impl SearchProvider for URLSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let all_recent = get_recent_entries("", 200);
        let url_history: Vec<_> = all_recent
            .into_iter()
            .filter(|e| e.name.starts_with("Open '"))
            .collect();

        let matcher = SkimMatcherV2::default();

        if q.is_empty() || !looks_like_url(q) {
            let mut scored: Vec<(i64, &crate::usage_tracker::UsageEntry)> = url_history
                .iter()
                .filter_map(|e| {
                    let term = term_from_name(&e.name);
                    if q.is_empty() {
                        Some((e.last_used as i64, e))
                    } else {
                        matcher.fuzzy_match(term, q).map(|s| (s, e))
                    }
                })
                .collect();
            scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));

            let results: Vec<ResultItem> = scored
                .iter()
                .take(8)
                .map(|(_, e)| {
                    let term = term_from_name(&e.name);
                    make_item(
                        e.name.clone(),
                        normalise_url(term),
                        format!("recent: {} time{}", e.count, if e.count == 1 { "" } else { "s" }),
                    )
                })
                .collect();

            return SearchResult {
                results,
                result_type: ResultType::List,
            };
        }

        let url = normalise_url(q);
        let primary = make_item(format!("Open '{}'", q), url, "Open URL");

        let mut scored: Vec<(i64, &crate::usage_tracker::UsageEntry)> = url_history
            .iter()
            .filter_map(|e| {
                let term = term_from_name(&e.name);
                matcher.fuzzy_match(term, q).map(|s| (s, e))
            })
            .collect();
        scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        let mut results = vec![primary];
        for (_, entry) in scored.iter().take(7) {
            let term = term_from_name(&entry.name);
            if term.eq_ignore_ascii_case(q) {
                continue; 
            }
            results.push(make_item(
                entry.name.clone(),
                normalise_url(term),
                format!("recent: {} time{}", entry.count, if entry.count == 1 { "" } else { "s" }),
            ));
        }

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
