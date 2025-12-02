use base64::{engine::general_purpose, Engine};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use once_cell::sync::Lazy;
use tauri::AppHandle;
use std::fs;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;

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
            None => continue, // Skip apps without exec
        };
        
        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());
        
        let icon = entry.icon().and_then(|i| resolve_icon(&i));
        
        apps.push(CachedApp {
            name,
            exec,
            description,
            icon,
        });
    }
    apps
});

pub struct AppSearcher;

impl SearchProvider for AppSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.to_lowercase();
        
        let mut results = Vec::new();
        
        for cached_app in APP_CACHE.iter() {
            if !cached_app.name.to_lowercase().contains(&q)
                && !cached_app
                    .description
                    .as_ref()
                    .map_or(false, |d| d.to_lowercase().contains(&q))
                && !cached_app.exec.to_lowercase().contains(&q)
            {
                continue;
            }
            
            let action_id = format!("app_{}", cached_app.name);
            
            let parts: Vec<String> = cached_app.exec
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            
            let executable = parts.first().cloned().unwrap_or_default();
            let args = parts.into_iter().skip(1).collect();
            
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::LaunchApp {
                        executable,
                        args,
                    },
                );
            }
            
            results.push(ResultItem {
                name: cached_app.name.clone(),
                action_id,
                description: cached_app.description.clone(),
                icon: cached_app.icon.clone(),
            });
        }
        
        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
