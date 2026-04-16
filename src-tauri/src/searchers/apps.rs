use base64::{engine::general_purpose, Engine};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use once_cell::sync::Lazy;
use tauri::AppHandle;
use std::fs;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};

fn clean_exec_field(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolve_icon(icon_name: &str) -> Option<String> {
    let path = freedesktop_icons::lookup(icon_name).with_size(64).find()?;
    let bytes = fs::read(&path).ok()?;
    let mime = if path.extension()?.to_str()? == "svg" {
        "image/svg+xml"
    } else {
        "image/png"
    };
    let encoded = general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, encoded))
}

#[derive(Clone)]
struct CachedApp {
    name: String,
    exec: String,
    description: Option<String>,
    icon: Option<String>,
}

static APP_CACHE: Lazy<Vec<CachedApp>> = Lazy::new(|| {
    let mut apps = Vec::new();
    let locales = get_languages_from_env();
    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    for entry in entries {
        if entry.no_display() || entry.hidden() {
            continue;
        }

        let name = entry
            .name(&locales)
            .or_else(|| entry.name::<&str>(&[]))
            .unwrap_or("Unknown".into())
            .to_string();

        let exec = match entry.exec() {
            Some(e) => clean_exec_field(&e),
            None => continue,
        };

        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());

        let icon = entry.icon().and_then(|i| resolve_icon(&i));

        apps.push(CachedApp { name, exec, description, icon });
    }
    apps
});

pub struct AppSearcher;

impl SearchProvider for AppSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let candidates: Vec<ResultItem> = APP_CACHE
            .iter()
            .map(|cached_app| {
                let parts: Vec<String> = cached_app.exec
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                let executable = parts.first().cloned().unwrap_or_default();
                let args = parts.into_iter().skip(1).collect();

                let mut item = ResultItem::new(
                    cached_app.name.clone(),
                    vec![ActionData::LaunchApp { executable, args }],
                );
                if let Some(desc) = &cached_app.description {
                    item = item.description(desc.clone());
                }
                if let Some(icon) = &cached_app.icon {
                    item = item.icon(icon.clone());
                }
                item
            })
            .collect();

        if query.is_empty() {
            let recent = crate::usage_tracker::get_recent_entries("", 30);
            if !recent.is_empty() {
                let mut name_map: std::collections::HashMap<String, ResultItem> = candidates
                    .into_iter()
                    .map(|item| (item.name.to_lowercase(), item))
                    .collect();

                let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut results: Vec<ResultItem> = recent
                    .iter()
                    .filter_map(|e| {
                        let key = e.name.to_lowercase();
                        if seen.contains(&key) {
                            return None;
                        }
                        if let Some(item) = name_map.remove(&key) {
                            seen.insert(key);
                            Some(item)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut remaining: Vec<ResultItem> = name_map.into_values().collect();
                remaining.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                results.extend(remaining);

                return SearchResult {
                    results,
                    result_type: ResultType::List,
                };
            }

            return SearchResult {
                results: candidates,
                result_type: ResultType::List,
            };
        }

        let results = self.fuzzy_filter(candidates, query);

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
