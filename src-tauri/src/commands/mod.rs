use regex::Regex;
use base64::{engine::general_purpose, Engine};
use once_cell::sync::Lazy;
use tauri::Manager;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::executor;
use crate::searchers::SearchProvider;
use crate::searchers::ai::AiSearcher;
use crate::searchers::apps::AppSearcher;
use crate::searchers::bookmarks::BookmarksSearcher;
use crate::searchers::camera::CameraSearcher;
use crate::searchers::screenshots::ScreenshotsSearcher;
use crate::searchers::shortcuts::ShortcutsSearcher;
use crate::searchers::scripts::ScriptsSearcher;
use crate::searchers::qrcode::QrCodeSearcher;
use crate::searchers::timer::TimerSearcher;
use crate::searchers::clipboard::ClipboardSearcher;
use crate::searchers::colorpicker::ColorPicker;
use crate::searchers::currency::CurrencySearcher;
use crate::searchers::dictionary::DictionarySearcher;
use crate::searchers::emojis::EmojiSearcher;
use crate::searchers::files::FileSearcher;
use crate::searchers::lorem::LoremSearcher;
use crate::searchers::math::MathSearcher;
use crate::searchers::note::NoteSearcher;
use crate::searchers::shell::ShellSearcher;
use crate::searchers::system::SystemSearcher;
use crate::searchers::time::TimeSearcher;
use crate::searchers::web_searchers::{URLSearcher, WebSearcher};
use crate::types::{ActionData, SearchResult};
use crate::usage_tracker::boost_results_by_usage;
use crate::{ACTION_REGISTRY, CLIPBOARD_MANAGER, CONFIG, SEARCH_SEQ, USAGE_HISTORY};

pub static TRIGGERS: Lazy<std::sync::RwLock<Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)>>> =
    Lazy::new(|| std::sync::RwLock::new(build_triggers(&CONFIG.read().unwrap())));

pub fn reload_triggers() {
    if let Ok(mut triggers) = TRIGGERS.write() {
        *triggers = build_triggers(&CONFIG.read().unwrap());
    }
}

// ---------------------------------------------------------
// TRIGGER BUILDER
// ---------------------------------------------------------

fn build_triggers(cfg: &config::Config) -> Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> {
    let t = &cfg.triggers;

    macro_rules! push {
        ($v:expr, $pattern:expr, $name:literal, $searcher:expr) => {
            match Regex::new($pattern) {
                Ok(r) => $v.push((r, Box::new($searcher) as Box<dyn SearchProvider + Send + Sync>)),
                Err(e) => eprintln!(
                    "quarry: invalid trigger regex for '{}' ({}) - skipping",
                    $name, e
                ),
            }
        };
    }

    let mut v: Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> = Vec::new();

    push!(v, &t.time, "time", TimeSearcher);
    push!(v, &t.ai, "ai", AiSearcher);
    push!(v, &t.camera, "camera", CameraSearcher);
    push!(v, &t.bookmarks, "bookmarks", BookmarksSearcher);
    push!(v, &t.files, "files", FileSearcher);
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
    push!(v, &t.shortcuts,   "shortcuts",   ShortcutsSearcher);
    push!(v, &t.scripts,     "scripts",     ScriptsSearcher);
    push!(v, &t.qr_code,     "qr_code",     QrCodeSearcher);

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
                Box::new(WebSearcher {
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

// ---------------------------------------------------------
// COMMANDS
// ---------------------------------------------------------

#[tauri::command]
pub async fn search(query: String, app: tauri::AppHandle) -> Option<SearchResult> {
    let my_seq = SEARCH_SEQ.fetch_add(1, Ordering::SeqCst) + 1;
    let query_clone = query.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut result = None;
        let triggers = TRIGGERS.read().unwrap();

        for (regex, searcher) in triggers.iter() {
            if let Some(caps) = regex.captures(&query_clone) {
                let rest = caps.get(1).map_or("", |m| m.as_str());
                let mut r = searcher.search(rest, &app);
                r.searcher = searcher.name().to_string();
                result = Some(r);
                break;
            }
        }

        result.unwrap_or_else(|| {
            use crate::searchers::default::DefaultSearcher;
            DefaultSearcher::new().search(&query_clone, &app)
        })
    })
    .await
    .ok()?;

    let mut search_result = result;

    if !matches!(search_result.result_type, crate::types::ResultType::Clipboard | crate::types::ResultType::Grid) {
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

    Some(search_result)
}

#[tauri::command]
pub fn execute(
    action_id: String,
    query: String,
    name: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let action_data = ACTION_REGISTRY
        .get_action(&action_id)
        .ok_or_else(|| format!("Action not found: {}", action_id))?;

    let tag = match &action_data {
        ActionData::CopyToClipboard { .. } | ActionData::CopyImageToClipboard { .. } => "copied",
        ActionData::RunFunction { function_name, .. } if function_name == "show_modal" => "stay",
        ActionData::RunFunction { function_name, .. } if function_name == "show_qr_clipboard" => "stay",
        ActionData::RunFunction { function_name, .. } if function_name == "copy_clipboard_image" => "copied",
        ActionData::RunFunction { function_name, .. } if function_name.starts_with("copy_") => "copied",
        ActionData::RunFunction { function_name, .. } if function_name == "trash_file" => "toasted:Moved to trash",
        ActionData::RunFunction { function_name, .. } if function_name == "pin" || function_name == "pin_clipboard_image" => "toasted:Pinned",
        ActionData::RunFunction { function_name, .. } if function_name == "unpin" => "toasted:Unpinned",
        _ => "launched",
    };

    let stable_id = action_data.stable_id();
    let result = executor::execute_action(action_data, &app);

    if result.is_ok() {
        if let Ok(mut history) = USAGE_HISTORY.write() {
            let display_name = name.as_deref().unwrap_or(&action_id);
            history.record_usage(&query, &stable_id, display_name);
        }
    }

    result.map(|_| tag.to_string())
}

#[tauri::command]
pub fn exec_func(name: String, params: Vec<String>, app: tauri::AppHandle) -> Result<(), String> {
    executor::execute_action(crate::types::ActionData::RunFunction { function_name: name, params }, &app)
}

#[tauri::command]
pub fn take_pending_query() -> Option<String> {
    crate::PENDING_QUERY.lock().ok().and_then(|mut slot| slot.take())
}

#[tauri::command]
pub fn get_theme() -> config::ThemeConfig {
    CONFIG.read().unwrap().theme.clone()
}

#[tauri::command]
pub fn get_ai_prefix() -> String {
    let pattern = CONFIG.read().unwrap().triggers.ai.clone();
    // Strip leading ^ and take everything before the first capture group
    let inner = pattern.trim_start_matches('^');
    let cap_start = inner.find('(').unwrap_or(inner.len());
    inner[..cap_start]
        .replace("\\s+", " ")
        .replace("\\s*", "")
        .replace("\\s", " ")
}

#[tauri::command]
pub fn get_config() -> config::Config {
    CONFIG.read().unwrap().clone()
}

#[tauri::command]
pub fn save_config(config: config::Config) -> Result<(), String> {
    config::Config::save(&config)?;
    if let Ok(mut c) = CONFIG.write() {
        *c = config;
    }
    reload_triggers();
    Ok(())
}

#[tauri::command]
pub fn get_clipboard_thumbnail(hash: String) -> Option<String> {
    let hash: u64 = hash.parse().ok()?;
    CLIPBOARD_MANAGER.get_thumbnail(hash)
}

#[tauri::command]
pub fn clear_clipboard_history() -> Result<(), String> {
    CLIPBOARD_MANAGER.clear_history();
    Ok(())
}

// ---------------------------------------------------------
// NOTE COMMANDS
// ---------------------------------------------------------

fn get_note_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("quarry")
        .join("scratch.txt")
}

#[tauri::command]
pub fn read_note() -> String {
    std::fs::read_to_string(get_note_path()).unwrap_or_default()
}

#[tauri::command]
pub fn write_note(content: String) -> Result<(), String> {
    let path = get_note_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------
// AI COMMANDS
// ---------------------------------------------------------

#[tauri::command]
pub fn get_groq_api_key() -> String {
    config::Config::load_groq_api_key()
}

#[tauri::command]
pub fn save_groq_api_key(key: String) -> Result<(), String> {
    config::Config::save_groq_api_key(&key)
}

// ---------------------------------------------------------
// ROFI COMMAND
// ---------------------------------------------------------

#[tauri::command]
pub fn cancel_rofi(response_socket: String) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    if let Ok(mut stream) = UnixStream::connect(&response_socket) {
        let _ = stream.write_all(b"\n");
    }
    Ok(())
}

// ---------------------------------------------------------
// DRAG COMMAND
// ---------------------------------------------------------

#[tauri::command]
pub async fn start_drag(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
    let win = window.clone();
    let win_cb = window.clone();

    window
        .app_handle()
        .run_on_main_thread(move || {
            // Drop always-on-top so the drop target is reachable
            let _ = win.set_always_on_top(false);

            let item = drag::DragItem::Files(vec![std::path::PathBuf::from(&path)]);
            let image = drag::Image::Raw(
                include_bytes!("../../../static/icons/file.png").to_vec(),
            );
            if let Ok(gtk_win) = win.gtk_window() {
                let _ = drag::start_drag(
                    &gtk_win,
                    item,
                    image,
                    move |result, _| match result {
                        drag::DragResult::Dropped => { let _ = win_cb.hide(); }
                        _ => { let _ = win_cb.set_always_on_top(true); }
                    },
                    Default::default(),
                );
            } else {
                let _ = win.set_always_on_top(true);
            }
        })
        .map_err(|e: tauri::Error| e.to_string())
}

// ---------------------------------------------------------
// CAPTURE COMMAND
// ---------------------------------------------------------

#[tauri::command]
pub async fn save_capture(png_base64: String) -> Result<String, String> {
    let pictures = dirs::picture_dir().ok_or("Could not find Pictures directory")?;

    let dir = pictures.join("quarry");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let path = dir.join(format!("{}-quarry-cam.png", ts));

    let bytes = general_purpose::STANDARD
        .decode(&png_base64)
        .map_err(|e| e.to_string())?;

    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().into_owned())
}
