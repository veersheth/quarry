use crate::searchers::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::usage_tracker::get_recent_entries;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tauri::AppHandle;

pub struct WebSearcher {
    pub name:         String,
    pub url_template: String,
    pub icon:         Option<String>,
}

impl WebSearcher {
    fn term_from_name<'a>(&self, name: &'a str) -> &'a str {
        let prefix = format!("Search {}:", self.name);
        name.strip_prefix(&prefix)
            .unwrap_or(name)
            .trim()
            .trim_matches('\'')
    }

    fn make_item(&self, name: String, term: &str, description: impl Into<String>) -> ResultItem {
        let url = self.url_template.replace("{}", &urlencoding::encode(term));
        let mut item = ResultItem::new(name, ActionData::OpenUrl { url })
            .description(description);
        if let Some(icon) = &self.icon {
            item = item.icon(icon.clone());
        }
        item
    }
}

impl SearchProvider for WebSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        let name_prefix = format!("Search {}:", self.name);

        // primary result is the direct search query
        let primary = if q.is_empty() {
            let mut item = ResultItem::new(
                format!("Search {} …", self.name),
                ActionData::None,
            )
            .description(format!("Type to search {}", self.name));
            if let Some(icon) = &self.icon {
                item = item.icon(icon.clone());
            }
            item
        } else {
            self.make_item(
                format!("{} '{}'", name_prefix, q),
                q,
                "Open in browser",
            )
        };

        let all_recent = get_recent_entries("", 200);
        let engine_history: Vec<_> = all_recent
            .into_iter()
            .filter(|e| e.name.starts_with(&name_prefix))
            .collect();

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(i64, &crate::usage_tracker::UsageEntry)> = engine_history
            .iter()
            .filter_map(|e| {
                let term = self.term_from_name(&e.name);
                if q.is_empty() {
                    Some((e.last_used as i64, e))
                } else {
                    matcher.fuzzy_match(term, q).map(|s| (s, e))
                }
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        let mut results = vec![primary];
        for (_, entry) in scored.iter().take(7) {
            let term = self.term_from_name(&entry.name);
            if !q.is_empty() && term.eq_ignore_ascii_case(q) {
                continue;
            }
            results.push(self.make_item(
                entry.name.clone(),
                term,
                format!("recent: {} time{}", entry.count, if entry.count == 1 { "" } else { "s" }),
            ));
        }

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
