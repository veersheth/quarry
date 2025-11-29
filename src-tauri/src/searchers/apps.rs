use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::borrow::Cow;
use super::SearchProvider;

#[derive(Debug, serde::Serialize, Clone)]
pub struct ListItem {
    pub name: String,
    pub exec: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

fn clean_exec_field(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn get_apps() -> Vec<ListItem> {
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
            .unwrap_or(Cow::Borrowed("Unknown"))
            .to_string();

        let exec = entry.exec().map(clean_exec_field);

        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());

        apps.push(ListItem {
            name,
            exec,
            description,
            icon: None,
        });
    }

    apps
}

pub struct AppSearcher;

impl SearchProvider for AppSearcher {
    fn search(&self, query: &str) -> Vec<ListItem> {
        let items = get_apps();
        let q = query.to_lowercase();

        items
            .into_iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&q)
                    || item.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&q))
                    || item.exec.as_ref().map_or(false, |e| e.to_lowercase().contains(&q))
            })
            .collect()
    }
}

