use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{atomic::Ordering, Arc, RwLock};
use tauri::Emitter;

use crate::config;
use crate::searchers::ai::AiSearcher;
use crate::searchers::apps::AppSearcher;
use crate::searchers::bookmarks::BookmarksSearcher;
use crate::searchers::camera::CameraSearcher;
use crate::searchers::clipboard::ClipboardSearcher;
use crate::searchers::colorpicker::ColorPicker;
use crate::searchers::currency::CurrencySearcher;
use crate::searchers::dictionary::DictionarySearcher;
use crate::searchers::emojis::EmojiSearcher;
use crate::searchers::files::FileSearcher;
use crate::searchers::fonts::FontsSearcher;
use crate::searchers::lorem::LoremSearcher;
use crate::searchers::math::MathSearcher;
use crate::searchers::note::NoteSearcher;
use crate::searchers::qrcode::QrCodeSearcher;
use crate::searchers::screenshots::ScreenshotsSearcher;
use crate::searchers::scripts::ScriptsSearcher;
use crate::searchers::shell::ShellSearcher;
use crate::searchers::shortcuts::ShortcutsSearcher;
use crate::searchers::system::SystemSearcher;
use crate::searchers::time::TimeSearcher;
use crate::searchers::timer::TimerSearcher;
use crate::searchers::web_searchers::{URLSearcher, WebSearcher};
use crate::searchers::SearchProvider;
use crate::types::SearchResult;
use crate::usage_tracker::boost_results_by_usage;
use crate::{ACTION_REGISTRY, CONFIG, SEARCH_SEQ, USAGE_HISTORY};

type Trigger = (Regex, Arc<dyn SearchProvider + Send + Sync>);

static TRIGGERS: Lazy<RwLock<Arc<Vec<Trigger>>>> =
    Lazy::new(|| RwLock::new(Arc::new(build_triggers(&CONFIG.read().unwrap()))));

pub fn reload_triggers() {
    if let Ok(mut triggers) = TRIGGERS.write() {
        *triggers = Arc::new(build_triggers(&CONFIG.read().unwrap()));
    }
}

// ---------------------------------------------------------
// TRIGGER BUILDER
// ---------------------------------------------------------

fn build_triggers(cfg: &config::Config) -> Vec<Trigger> {
    let t = &cfg.triggers;

    macro_rules! push {
        ($v:expr, $pattern:expr, $name:literal, $searcher:expr) => {
            match Regex::new($pattern) {
                Ok(r) => $v.push((
                    r,
                    Arc::new($searcher) as Arc<dyn SearchProvider + Send + Sync>,
                )),
                Err(e) => eprintln!(
                    "quarry: invalid trigger regex for '{}' ({}) - skipping",
                    $name, e
                ),
            }
        };
    }

    let mut v: Vec<Trigger> = Vec::new();

    push!(v, &t.time, "time", TimeSearcher);
    push!(v, &t.ai, "ai", AiSearcher);
    push!(v, &t.camera, "camera", CameraSearcher);
    push!(v, &t.bookmarks, "bookmarks", BookmarksSearcher);
    push!(v, &t.files, "files", FileSearcher);
    push!(v, &t.fonts, "fonts", FontsSearcher);
    push!(v, &t.clipboard, "clipboard", ClipboardSearcher);
    push!(v, &t.emojis, "emojis", EmojiSearcher);
    push!(v, &t.shell, "shell", ShellSearcher);
    push!(v, &t.lorem, "lorem", LoremSearcher);
    push!(v, &t.math, "math", MathSearcher);
    push!(v, &t.dictionary, "dictionary", DictionarySearcher);
    push!(v, &t.system, "system", SystemSearcher);
    push!(v, &t.color_picker, "color_picker", ColorPicker);
    push!(v, &t.apps, "apps", AppSearcher);
    push!(v, &t.url, "url", URLSearcher);
    push!(v, &t.currency, "currency", CurrencySearcher);
    push!(v, &t.note, "note", NoteSearcher);
    push!(v, &t.timer, "timer", TimerSearcher);
    push!(v, &t.screenshots, "screenshots", ScreenshotsSearcher);
    push!(v, &t.shortcuts, "shortcuts", ShortcutsSearcher);
    push!(v, &t.scripts, "scripts", ScriptsSearcher);
    push!(v, &t.qr_code, "qr_code", QrCodeSearcher);

    // Detect raw pasted color values without requiring the "color" prefix.
    push!(
        v,
        r"^(#[0-9a-fA-F]{3,8}|rgba?\s*\([^)]+\)|hsla?\s*\([^)]+\)|oklch\s*\([^)]+\))$",
        "color_value",
        ColorPicker
    );

    for ws in &cfg.web_searches {
        match Regex::new(&ws.trigger) {
            Ok(r) => v.push((
                r,
                Arc::new(WebSearcher {
                    name: ws.name.clone(),
                    url_template: ws.url.clone(),
                    icon: ws.icon.clone(),
                }),
            )),
            Err(e) => eprintln!(
                "quarry: invalid trigger regex for web search '{}' ({}) - skipping",
                ws.name, e
            ),
        }
    }

    v
}

#[tauri::command]
pub async fn search(query: String, app: tauri::AppHandle) -> Option<SearchResult> {
    let my_seq = SEARCH_SEQ.fetch_add(1, Ordering::SeqCst) + 1;
    let query_clone = query.clone();
    let app_for_search = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut result = None;
        let triggers = Arc::clone(&TRIGGERS.read().unwrap());

        for (regex, searcher) in triggers.iter() {
            if let Some(caps) = regex.captures(&query_clone) {
                let rest = caps.get(1).map_or("", |m| m.as_str());
                let mut r = searcher.search(rest, &app_for_search);
                r.searcher = searcher.name().to_string();
                result = Some(r);
                break;
            }
        }

        result.unwrap_or_else(|| {
            use crate::searchers::default::DefaultSearcher;
            DefaultSearcher.search(&query_clone, &app_for_search)
        })
    })
    .await
    .ok()?;

    let mut search_result = result;

    if !matches!(
        search_result.result_type,
        crate::types::ResultType::Clipboard | crate::types::ResultType::Grid
    ) {
        if let Ok(history) = USAGE_HISTORY.read() {
            search_result.results = boost_results_by_usage(search_result.results, &query, &history);
        }
    }

    if SEARCH_SEQ.load(Ordering::SeqCst) != my_seq {
        return None;
    }

    ACTION_REGISTRY.clear();
    for (i, item) in search_result.results.iter_mut().enumerate() {
        for (j, action) in item.actions.iter_mut().enumerate() {
            action.id = format!("action_{}_{}_{}", my_seq, i, j);
            ACTION_REGISTRY.register(action.id.clone(), action.data.clone());
        }
    }

    search_result.seq = my_seq;

    // For clipboard results: return the first 30 immediately and emit the rest
    // in a background task so the UI shows something instantly.
    if matches!(
        search_result.result_type,
        crate::types::ResultType::Clipboard
    ) && search_result.results.len() > 30
    {
        let rest: Vec<_> = search_result.results.drain(30..).collect();
        let app_for_more = app.clone();
        tauri::async_runtime::spawn(async move {
            // Brief yield — lets the IPC response for the first 30 items land
            // before the event carrying the rest arrives.
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            if SEARCH_SEQ.load(Ordering::SeqCst) == my_seq {
                #[derive(serde::Serialize, Clone)]
                struct ClipboardMore {
                    seq: u64,
                    results: Vec<crate::types::ResultItem>,
                }
                let _ = app_for_more.emit(
                    "quarry-clipboard-more",
                    ClipboardMore {
                        seq: my_seq,
                        results: rest,
                    },
                );
            }
        });
    }

    Some(search_result)
}

pub fn warm_triggers() {
    Lazy::force(&TRIGGERS);
}
