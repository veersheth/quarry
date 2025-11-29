use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::borrow::Cow;

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

        let icon = None;

        apps.push(ListItem {
            name,
            exec,
            description,
            icon,
        });
    }

    apps
}

