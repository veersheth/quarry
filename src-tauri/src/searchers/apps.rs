use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

fn clean_exec_field(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolve_icon(icon_name: &str) -> Option<String> {
    let path = freedesktop_icons::lookup(icon_name).with_size(64).find()?;
    path.to_str().map(|s| s.to_string())
}

#[derive(Clone)]
struct AppAction {
    name: String,
    exec: String,
    icon: Option<String>,
}

#[derive(Clone)]
struct CachedApp {
    name: String,
    exec: String,
    description: Option<String>,
    icon: Option<String>,
    app_actions: Vec<AppAction>,
}

static APP_CACHE: Lazy<RwLock<Vec<CachedApp>>> = Lazy::new(|| RwLock::new(build_app_list()));

fn build_app_list() -> Vec<CachedApp> {
    let mut apps = Vec::new();
    let locales = get_languages_from_env();
    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    for entry in &entries {
        if entry.no_display() || entry.hidden() {
            continue;
        }

        let name = match entry.name(&locales).or_else(|| entry.name::<&str>(&[])) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let exec = match entry.exec() {
            Some(e) => clean_exec_field(&e),
            None => continue,
        };

        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());

        let icon = entry.icon().and_then(|i| resolve_icon(&i));

        let app_actions: Vec<AppAction> = entry
            .actions()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|action_id| {
                let name = entry.action_entry(action_id, "Name")?.to_string();
                let exec_raw = entry.action_entry(action_id, "Exec")?;
                let exec = clean_exec_field(exec_raw);
                let action_icon = entry
                    .action_entry(action_id, "Icon")
                    .and_then(resolve_icon)
                    .or_else(|| icon.clone());
                Some(AppAction { name, exec, icon: action_icon })
            })
            .collect();

        apps.push(CachedApp { name, exec, description, icon, app_actions });
    }

    apps
}

pub fn warm() {
    let _ = APP_CACHE.read().map(|g| g.len());
}

pub fn reload() {
    if let Ok(mut cache) = APP_CACHE.write() {
        *cache = build_app_list();
    }
}

pub struct AppSearcher;

fn launch_actions(exec: &str) -> Vec<Action> {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    let executable = parts.first().copied().unwrap_or("").to_string();
    let args: Vec<String> = parts.iter().skip(1).map(|s| s.to_string()).collect();
    vec![Action::new("Launch", ActionData::LaunchApp { executable, args })]
}

fn build_result_item(app: &CachedApp) -> ResultItem {
    let parts: Vec<&str> = app.exec.split_whitespace().collect();
    let executable = parts.first().copied().unwrap_or("").to_string();
    let args: Vec<String> = parts.iter().skip(1).map(|s| s.to_string()).collect();

    let mut actions = vec![
        Action::new("Launch", ActionData::LaunchApp { executable: executable.clone(), args }),
    ];

    for app_action in &app.app_actions {
        let mut launch = launch_actions(&app_action.exec);
        let mut a = launch.remove(0);
        a = Action::new(app_action.name.clone(), a.data);
        actions.push(a);
    }

    actions.push(Action::new("Run as Administrator", ActionData::ShellCommand {
        command: format!("pkexec {}", app.exec),
    }));
    actions.push(Action::new("Copy Command", ActionData::CopyToClipboard { text: app.exec.clone() }));
    actions.push(Action::new("Copy Executable Path", ActionData::CopyToClipboard { text: executable }));

    let mut item = ResultItem::new(app.name.clone(), actions);
    if let Some(desc) = &app.description { item = item.description(desc.clone()); }
    if let Some(icon) = &app.icon { item = item.icon(icon.clone()); }
    item
}


fn fuzzy_score_app(matcher: &SkimMatcherV2, app: &CachedApp, query: &str) -> i64 {
    let name_score = matcher.fuzzy_match(&app.name, query).unwrap_or(0) * 2;
    let desc_score = app.description.as_deref()
        .and_then(|d| matcher.fuzzy_match(d, query))
        .unwrap_or(0);
    name_score.max(desc_score)
}

impl SearchProvider for AppSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let guard = APP_CACHE.read().unwrap_or_else(|e| e.into_inner());

        if query.is_empty() {
            let recent = crate::usage_tracker::get_recent_entries("", 30);

            if !recent.is_empty() {
                let mut name_map: std::collections::HashMap<String, &CachedApp> = guard
                    .iter()
                    .map(|a| (a.name.to_lowercase(), a))
                    .collect();

                let mut seen = std::collections::HashSet::new();
                let mut results: Vec<ResultItem> = recent
                    .iter()
                    .filter_map(|e| {
                        let key = e.name.to_lowercase();
                        if seen.contains(&key) { return None; }
                        if let Some(app) = name_map.remove(&key) {
                            seen.insert(key);
                            Some(build_result_item(app))
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut remaining: Vec<ResultItem> = name_map.into_values()
                    .map(build_result_item)
                    .collect();
                remaining.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                results.extend(remaining);

                return SearchResult { results, result_type: ResultType::List };
            }

            let results = guard.iter().map(build_result_item).collect();
            return SearchResult { results, result_type: ResultType::List };
        }

        let matcher = SkimMatcherV2::default().ignore_case();
        let mut scored: Vec<(&CachedApp, i64)> = guard
            .iter()
            .filter_map(|app| {
                let score = fuzzy_score_app(&matcher, app, query);
                if score > 0 { Some((app, score)) } else { None }
            })
            .collect();
        scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let mut results: Vec<ResultItem> = scored
            .iter()
            .map(|(app, _)| build_result_item(app))
            .collect();

        SearchResult { results, result_type: ResultType::List }
    }
}
