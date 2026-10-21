use base64::Engine;
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use once_cell::sync::Lazy;
use std::path::Path;
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

static ICON_THEME: Lazy<Option<String>> = Lazy::new(|| {
    // GTK settings files are commonly used to configure icon themes on
    // non-GNOME desktops (including wlroots compositors). Prefer those over
    // gsettings, which may report its default theme even when GTK is themed
    // through settings.ini.
    let config_dir = dirs::config_dir()?;
    for gtk_version in ["gtk-4.0", "gtk-3.0"] {
        let settings = config_dir.join(gtk_version).join("settings.ini");
        if let Some(theme) = read_gtk_icon_theme(&settings) {
            return Some(theme);
        }
    }

    freedesktop_icons::default_theme_gtk()
});

fn read_gtk_icon_theme(path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(path).ok()?;
    let mut in_settings = false;
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_settings = line == "[Settings]";
        } else if in_settings {
            if let Some(value) = line.strip_prefix("gtk-icon-theme-name=") {
                let theme = value
                    .trim()
                    .trim_matches(|character| character == '\"' || character == '\'');
                if !theme.is_empty() {
                    return Some(theme.to_string());
                }
            }
        }
    }
    None
}

fn resolve_icon(icon_name: &str) -> Option<String> {
    let path = if icon_name.starts_with('/') {
        Path::new(icon_name)
            .exists()
            .then(|| Path::new(icon_name).to_path_buf())
    } else {
        let lookup = freedesktop_icons::lookup(icon_name).with_size(64);
        match ICON_THEME.as_deref() {
            Some(theme) => lookup.with_theme(theme).find(),
            None => lookup.find(),
        }
    }?;

    // Tauri's asset:// URLs are cross-origin from the http://localhost dev
    // server in Linux WebKitGTK. Embed the small app icon instead, so WebKit
    // can load it without relying on the asset protocol or its path scope.
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let mime = match extension.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "xpm" => "image/x-xpixmap",
        _ => return None,
    };
    let bytes = std::fs::read(path).ok()?;
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
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
            Some(e) => clean_exec_field(e),
            None => continue,
        };

        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());

        let icon = entry.icon().and_then(resolve_icon);

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
                Some(AppAction {
                    name,
                    exec,
                    icon: action_icon,
                })
            })
            .collect();

        apps.push(CachedApp {
            name,
            exec,
            description,
            icon,
            app_actions,
        });
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
    vec![Action::new(
        "Launch",
        ActionData::LaunchApp { executable, args },
    )]
}

fn preferred_terminal() -> String {
    std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".to_string())
}

fn terminal_launch(exec: &str) -> Action {
    Action::new(
        "Run in Terminal",
        ActionData::LaunchApp {
            executable: preferred_terminal(),
            args: vec!["-e".to_string(), exec.to_string()],
        },
    )
}

fn copy_actions(exec: &str) -> [Action; 2] {
    let executable = exec.split_whitespace().next().unwrap_or("").to_string();
    [
        Action::new(
            "Copy Command",
            ActionData::CopyToClipboard {
                text: exec.to_string(),
            },
        ),
        Action::new(
            "Copy Executable",
            ActionData::CopyToClipboard { text: executable },
        ),
    ]
}

fn build_result_item(app: &CachedApp) -> ResultItem {
    let parts: Vec<&str> = app.exec.split_whitespace().collect();
    let executable = parts.first().copied().unwrap_or("").to_string();
    let args: Vec<String> = parts.iter().skip(1).map(|s| s.to_string()).collect();

    let mut actions = vec![Action::new(
        "Launch",
        ActionData::LaunchApp {
            executable: executable.clone(),
            args,
        },
    )];

    for app_action in &app.app_actions {
        let mut launch = launch_actions(&app_action.exec);
        let mut a = launch.remove(0);
        a = Action::new(app_action.name.clone(), a.data);
        actions.push(a);
    }

    actions.push(terminal_launch(&app.exec));
    actions.push(Action::new(
        "Run as Administrator",
        ActionData::LaunchApp {
            executable: "pkexec".to_string(),
            args: {
                let display = std::env::var("DISPLAY").unwrap_or_default();
                let wayland = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
                let xdg_rt = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
                let path = std::env::var("PATH").unwrap_or_default();
                let mut a = vec![
                    "env".to_string(),
                    format!("DISPLAY={}", display),
                    format!("WAYLAND_DISPLAY={}", wayland),
                    format!("XDG_RUNTIME_DIR={}", xdg_rt),
                    format!("PATH={}", path),
                ];
                a.extend(app.exec.split_whitespace().map(|s| s.to_string()));
                a
            },
        },
    ));
    actions.extend(copy_actions(&app.exec));

    let mut item = ResultItem::new(app.name.clone(), actions);
    if let Some(desc) = &app.description {
        item = item.description(desc.clone());
    }
    if let Some(icon) = &app.icon {
        item = item.icon(icon.clone());
    }
    item
}

fn fuzzy_score_app(app: &CachedApp, query: &str) -> i64 {
    let mut fields = vec![app.name.as_str()];
    if let Some(description) = app.description.as_deref() {
        fields.push(description);
    }

    let Some(all_fields_score) = crate::search_utils::match_all_tokens(&fields, query) else {
        return 0;
    };
    let name_score =
        crate::search_utils::match_all_tokens(&[app.name.as_str()], query).unwrap_or(0);
    all_fields_score + name_score
}

impl SearchProvider for AppSearcher {
    fn name(&self) -> String {
        "apps".to_string()
    }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let guard = APP_CACHE.read().unwrap_or_else(|e| e.into_inner());

        if query.is_empty() {
            let recent = crate::usage_tracker::get_recent_entries("", 30);

            if !recent.is_empty() {
                let mut name_map: std::collections::HashMap<String, &CachedApp> =
                    guard.iter().map(|a| (a.name.to_lowercase(), a)).collect();

                let mut seen = std::collections::HashSet::new();
                let mut results: Vec<ResultItem> = recent
                    .iter()
                    .filter_map(|e| {
                        let key = e.name.to_lowercase();
                        if seen.contains(&key) {
                            return None;
                        }
                        if let Some(app) = name_map.remove(&key) {
                            seen.insert(key);
                            Some(build_result_item(app))
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut remaining: Vec<ResultItem> =
                    name_map.into_values().map(build_result_item).collect();
                remaining.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                results.extend(remaining);

                return SearchResult {
                    results,
                    result_type: ResultType::Apps,
                    ..Default::default()
                };
            }

            let results = guard.iter().map(build_result_item).collect();
            return SearchResult {
                results,
                result_type: ResultType::Apps,
                ..Default::default()
            };
        }

        // Score apps and their actions together, then interleave by score.
        enum ScoredItem<'a> {
            App(&'a CachedApp, i64),
            Action(&'a CachedApp, &'a AppAction, i64),
        }

        let mut scored: Vec<ScoredItem> = Vec::new();

        for app in guard.iter() {
            let app_score = fuzzy_score_app(app, query);
            if app_score > 0 {
                scored.push(ScoredItem::App(app, app_score));
            }

            for action in &app.app_actions {
                // Score against "App Name Action Name" and plain action name.
                let combined = format!("{} {}", app.name, action.name);
                let action_score = crate::search_utils::match_all_tokens(
                    &[combined.as_str(), action.name.as_str()],
                    query,
                )
                .unwrap_or(0);
                if action_score > 0 {
                    scored.push(ScoredItem::Action(app, action, action_score));
                }
            }
        }

        scored.sort_unstable_by(|a, b| {
            let sa = match a {
                ScoredItem::App(_, s) | ScoredItem::Action(_, _, s) => *s,
            };
            let sb = match b {
                ScoredItem::App(_, s) | ScoredItem::Action(_, _, s) => *s,
            };
            sb.cmp(&sa)
        });

        let results = scored
            .iter()
            .map(|s| match s {
                ScoredItem::App(app, _) => build_result_item(app),
                ScoredItem::Action(app, action, _) => {
                    let mut actions = launch_actions(&action.exec);
                    actions.push(terminal_launch(&action.exec));
                    actions.extend(copy_actions(&action.exec));
                    let mut item = ResultItem::new(action.name.clone(), actions);
                    item = item.description(format!("{} - {}", app.name, action.name));
                    if let Some(icon) = &action.icon {
                        item = item.icon(icon.clone());
                    }
                    item
                }
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::Apps,
            ..Default::default()
        }
    }
}
