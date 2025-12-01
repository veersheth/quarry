use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use freedesktop_icons::lookup;
use std::path::PathBuf;

use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult};

fn clean_exec_field(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

use std::{fs, path::Path};
use base64::{engine::general_purpose, Engine};

fn resolve_icon(icon_name: &str) -> Option<String> {
    use std::path::PathBuf;

    // 1. absolute path: freedesktop-icons already returns this
    let path = freedesktop_icons::lookup(icon_name)
        .with_size(64)
        .find()?;

    // 2. read file as bytes
    let bytes = fs::read(&path).ok()?;

    // 3. detect extension
    let mime = if path.extension()?.to_str()? == "svg" {
        "image/svg+xml"
    } else {
        "image/png"
    };

    // 4. convert to data URL
    let encoded = general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, encoded))
}

pub fn get_apps() -> Vec<ResultItem> {
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

        let exec = entry.exec().map(clean_exec_field);

        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());

        // NEW: resolve icon path
        let icon = entry.icon().and_then(|i| resolve_icon(&i));

        apps.push(ResultItem {
            name,
            exec,
            description,
            icon,
        });
    }

    apps
}

pub struct AppSearcher;

impl SearchProvider for AppSearcher {
    fn search(&self, query: &str) -> SearchResult {
        let items = get_apps();
        let q = query.to_lowercase();

        let results = items
            .into_iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&q)
                    || item
                        .description
                        .as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&q))
                    || item
                        .exec
                        .as_ref()
                        .map_or(false, |e| e.to_lowercase().contains(&q))
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
