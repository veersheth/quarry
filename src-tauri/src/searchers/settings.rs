use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use once_cell::sync::Lazy;
use std::sync::RwLock;
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

#[derive(Clone)]
struct SettingsEntry {
    name: String,
    exec: String,
    description: Option<String>,
    icon: Option<String>,
}

static SETTINGS_CACHE: Lazy<RwLock<Vec<SettingsEntry>>> =
    Lazy::new(|| RwLock::new(build_settings_list()));

fn build_settings_list() -> Vec<SettingsEntry> {
    let locales = get_languages_from_env();
    let mut entries: Vec<SettingsEntry> = Iter::new(default_paths())
        .entries(Some(&locales))
        .filter(|e| {
            if e.hidden() {
                return false;
            }
            e.categories()
                .map(|cats| cats.iter().any(|c| c.trim() == "Settings"))
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let exec = entry.exec()?.to_string();
            let exec = exec
                .split_whitespace()
                .filter(|p| !p.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ");

            let name = entry
                .name(&locales)
                .or_else(|| entry.name::<&str>(&[]))?
                .to_string();

            let description = entry
                .comment(&locales)
                .or_else(|| entry.comment::<&str>(&[]))
                .map(|s| s.to_string());

            let icon = entry
                .icon()
                .and_then(|i| freedesktop_icons::lookup(&i).with_size(64).find())
                .and_then(|p| p.to_str().map(|s| s.to_string()));

            Some(SettingsEntry { name, exec, description, icon })
        })
        .collect();

    entries.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    entries.dedup_by(|a, b| a.name == b.name);
    entries
}

pub struct SettingsSearcher;

impl SearchProvider for SettingsSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let guard = SETTINGS_CACHE.read().unwrap_or_else(|e| e.into_inner());

        let candidates: Vec<ResultItem> = guard
            .iter()
            .map(|entry| {
                let parts: Vec<String> = entry
                    .exec
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                let executable = parts.first().cloned().unwrap_or_default();
                let args: Vec<String> = parts.into_iter().skip(1).collect();

                let mut item = ResultItem::new(
                    entry.name.clone(),
                    vec![Action::new(
                        "Open",
                        ActionData::LaunchApp { executable, args },
                    )],
                );
                if let Some(desc) = &entry.description {
                    item = item.description(desc.clone());
                }
                if let Some(icon) = &entry.icon {
                    item = item.icon(icon.clone());
                }
                item
            })
            .collect();

        drop(guard);

        let results = if query.is_empty() {
            candidates
        } else {
            self.fuzzy_filter(candidates, query)
        };

        SearchResult { results, result_type: ResultType::List }
    }
}
