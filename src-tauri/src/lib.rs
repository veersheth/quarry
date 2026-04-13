mod action_registry;
mod clipboard_manager;
mod config;
mod ipc_server;
mod searchers;
mod types;
mod usage_tracker;

use crate::searchers::ai::AiSearcher;
use crate::searchers::currency::CurrencySearcher;
use crate::searchers::camera::CameraSearcher;
use crate::searchers::note::NoteSearcher;
use crate::searchers::bookmarks::BookmarksSearcher;
use crate::searchers::files::FileSearcher;
use crate::searchers::clipboard::ClipboardSearcher;
use crate::searchers::colorpicker::ColorPicker;
use crate::searchers::lorem::LoremSearcher;
use crate::searchers::shell::ShellSearcher;
use crate::searchers::system::SystemSearcher;
use crate::searchers::web_searchers::{URLSearcher, WebSearcher};
use searchers::apps::AppSearcher;
use searchers::dictionary::DictionarySearcher;
use searchers::emojis::EmojiSearcher;
use searchers::math::MathSearcher;
use searchers::SearchProvider;

use action_registry::ActionRegistry;
use clipboard_manager::ClipboardManager;
use tauri::{Emitter, Manager, command};
use types::{ActionData, SearchResult};
use usage_tracker::{boost_results_by_usage, UsageHistory};

use base64::{engine::general_purpose, Engine};
use lazy_static::lazy_static;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_cli::CliExt;

// ---------------------------------------------------------
// GLOBAL STATE
// ---------------------------------------------------------

static SEARCH_SEQ: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    pub static ref CONFIG: config::Config = {
        config::Config::write_default_if_missing();
        config::Config::load()
    };

    pub static ref USAGE_HISTORY: RwLock<UsageHistory> = RwLock::new(UsageHistory::load());
    static ref ACTION_REGISTRY: ActionRegistry = ActionRegistry::new();

    static ref CLIPBOARD_MANAGER: ClipboardManager = {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("tauri");
        std::fs::create_dir_all(&data_dir).ok();
        let clipboard_path = data_dir.join("clipboard_history.json");
        ClipboardManager::with_storage(clipboard_path)
    };
}

// ---------------------------------------------------------
// TRIGGER BUILDER
// ---------------------------------------------------------
fn build_triggers() -> Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> {
    let cfg = config::Config::load();
    let t = &cfg.triggers;

    macro_rules! push {
        ($v:expr, $pattern:expr, $name:literal, $searcher:expr) => {
            match Regex::new($pattern) {
                Ok(r) => $v.push((r, Box::new($searcher) as Box<dyn SearchProvider + Send + Sync>)),
                Err(e) => eprintln!(
                    "quarry: invalid trigger regex for '{}' ({}) — skipping",
                    $name, e
                ),
            }
        };
    }

    let mut v: Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> = Vec::new();

    push!(v, &t.ai,           "ai",           AiSearcher);
    push!(v, &t.camera,       "camera",       CameraSearcher);
    push!(v, &t.bookmarks,    "bookmarks",    BookmarksSearcher);
    push!(v, &t.files,        "files",        FileSearcher);
    push!(v, &t.clipboard,    "clipboard",    ClipboardSearcher);
    push!(v, &t.emojis,       "emojis",       EmojiSearcher);
    push!(v, &t.shell,        "shell",        ShellSearcher);
    push!(v, &t.lorem,        "lorem",        LoremSearcher);
    push!(v, &t.math,         "math",         MathSearcher);
    push!(v, &t.dictionary,   "dictionary",   DictionarySearcher);
    push!(v, &t.system,       "system",       SystemSearcher);
    push!(v, &t.color_picker, "color_picker", ColorPicker);
    push!(v, &t.apps,         "apps",         AppSearcher);
    push!(v, &t.url,          "url",          URLSearcher);
    push!(v, &t.currency,     "currency",     CurrencySearcher);
    push!(v, &t.note,         "note",         NoteSearcher);

    for ws in &cfg.web_searches {
        match Regex::new(&ws.trigger) {
            Ok(r) => v.push((
                r,
                Box::new(WebSearcher {
                    name:         ws.name.clone(),
                    url_template: ws.url.clone(),
                    icon:         ws.icon.clone(),
                }),
            )),
            Err(e) => eprintln!(
                "quarry: invalid trigger regex for web search '{}' ({}) — skipping",
                ws.name, e
            ),
        }
    }

    v
}

// ---------------------------------------------------------
// SEARCH COMMAND
// ---------------------------------------------------------
#[tauri::command]
async fn search(query: String, app: tauri::AppHandle) -> Option<SearchResult> {
    let my_seq = SEARCH_SEQ.fetch_add(1, Ordering::SeqCst) + 1;

    let query_clone = query.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let triggers = build_triggers();
        let mut result = None;

        for (regex, searcher) in triggers.iter() {
            if let Some(caps) = regex.captures(&query_clone) {
                let rest = caps.get(1).map_or("", |m| m.as_str());
                result = Some(searcher.search(rest, &app));
                break;
            }
        }

        result.unwrap_or_else(|| {
            use searchers::default::DefaultSearcher;
            DefaultSearcher::new().search(&query_clone, &app)
        })
    })
    .await
    .ok()?;

    let mut search_result = result;

    for (i, item) in search_result.results.iter_mut().enumerate() {
        let id = format!("action_{}_{}", my_seq, i);
        ACTION_REGISTRY.register(id.clone(), item.action.clone());
        item.action_id = id;
    }

    if let Ok(history) = USAGE_HISTORY.read() {
        search_result.results =
            boost_results_by_usage(search_result.results, &query, &history);
    }

    if SEARCH_SEQ.load(Ordering::SeqCst) != my_seq {
        return None;
    }

    Some(search_result)
}

// ---------------------------------------------------------
// THEME COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn get_theme() -> config::ThemeConfig {
    config::Config::load().theme
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
fn read_note() -> String {
    std::fs::read_to_string(get_note_path()).unwrap_or_default()
}

#[tauri::command]
fn write_note(content: String) -> Result<(), String> {
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
fn get_groq_api_key() -> String {
    config::Config::load().groq_api_key
}

#[tauri::command]
fn save_groq_api_key(key: String) -> Result<(), String> {
    config::Config::save_groq_api_key(&key)
}

// ---------------------------------------------------------
// EXECUTE COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn execute(action_id: String, query: String, name: Option<String>, app: tauri::AppHandle) -> Result<(), String> {
    let action_data = ACTION_REGISTRY
        .get_action(&action_id)
        .ok_or_else(|| format!("Action not found: {}", action_id))?;

    let result = match action_data {
        ActionData::LaunchApp { executable, args } => launch_app(&executable, &args),
        ActionData::OpenUrl { url } => open_url(&url, &app),
        ActionData::CopyToClipboard { text } => copy_to_clipboard(&text, &app),
        ActionData::CopyImageToClipboard { base64_png, width, height } => {
            copy_image_to_clipboard(&base64_png, width, height)
        }
        ActionData::RunFunction { function_name, params } => {
            run_custom_function(&function_name, &params, &app)
        }
        ActionData::ShellCommand { command } => run_shell_command(&command),
        ActionData::None => Ok(()),
    };

    if result.is_ok() {
        if let Ok(mut history) = USAGE_HISTORY.write() {
            let display_name = name.as_deref().unwrap_or(&action_id);
            history.record_usage(&query, &action_id, display_name);
        }
    }

    result
}

// ---------------------------------------------------------
// CLIPBOARD COMMANDS
// ---------------------------------------------------------
#[tauri::command]
fn clear_clipboard_history() -> Result<(), String> {
    CLIPBOARD_MANAGER.clear_history();
    Ok(())
}

// ---------------------------------------------------------
// CAPTURE COMMAND
// ---------------------------------------------------------
#[tauri::command]
async fn save_capture(png_base64: String) -> Result<String, String> {
    let pictures = dirs::picture_dir()
        .ok_or("Could not find Pictures directory")?;

    let dir = pictures.join("quarry");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let filename = format!("{}-quarry-cam.png", ts);
    let path = dir.join(&filename);

    let bytes = general_purpose::STANDARD
        .decode(&png_base64)
        .map_err(|e| e.to_string())?;

    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().into_owned())
}

// ---------------------------------------------------------
// EXECUTION HANDLERS
// ---------------------------------------------------------
fn launch_app(executable: &str, args: &[String]) -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    Command::new(executable)
        .args(args)
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", executable, e))?;
    Ok(())
}

fn open_url(url: &str, app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

fn copy_to_clipboard(text: &str, app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

fn copy_image_to_clipboard(base64_png: &str, width: u32, height: u32) -> Result<(), String> {
    let b64 = base64_png
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(base64_png);

    let png_bytes = general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("Failed to decode image base64: {}", e))?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to decode PNG: {}", e))?
        .into_rgba8();

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {}", e))?;

    let img_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: img.into_raw().into(),
    };

    clipboard
        .set_image(img_data)
        .map_err(|e| format!("Failed to set clipboard image: {}", e))?;

    Ok(())
}

fn run_shell_command(command: &str) -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to run shell command: {}", e))?;
    Ok(())
}

fn run_custom_function(
    function_name: &str,
    params: &[String],
    app: &tauri::AppHandle,
) -> Result<(), String> {
    match function_name {
        "open_note" => {
            if let Some(window) = app.get_webview_window("note") {
                let window = window.as_ref().window();
                let _ = window.show();
                let _ = window.set_focus();
            }
            Ok(())
        }
        "open_ai_chat" => {
            let query = params.first().cloned().unwrap_or_default();
            open_or_create_ai_window(app, &query)
        }
        "clear_clipboard" => {
            let _ = clear_clipboard_history();
            Ok(())
        }
        "add_bookmark" => {
            if params.len() != 2 {
                return Err("add_bookmark requires name and url".to_string());
            }
            BookmarksSearcher::add_bookmark(params[0].clone(), params[1].clone()).map(|_| ())
        }
        "remove_bookmark" => {
            if params.is_empty() {
                return Err("remove_bookmark requires a name".to_string());
            }
            BookmarksSearcher::remove_bookmark(&params[0]).map(|_| ())
        }
        _ => Err(format!("Unknown function: {}", function_name)),
    }
}

fn open_or_create_ai_window(app: &tauri::AppHandle, query: &str) -> Result<(), String> {
    let encoded_query = urlencoding::encode(query).to_string();
    let url = format!("ai?q={}", encoded_query);

    if let Some(window) = app.get_webview_window("ai") {
        let window_ref = window.as_ref().window();
        // Push the new query into the already-open window via JS
        if !query.is_empty() {
            let js = format!(
                "window.__quarryNewQuery && window.__quarryNewQuery({})",
                serde_json::to_string(query).unwrap_or_default()
            );
            let _ = window.eval(&js);
        }
        let _ = window_ref.show();
        let _ = window_ref.set_focus();
        return Ok(());
    }

    // Window doesn't exist yet — create it
    let ai_webview = tauri::WebviewWindowBuilder::new(
        app,
        "ai",
        tauri::WebviewUrl::App(url.into()),
    )
    .title("quarry ai")
    .inner_size(680.0, 520.0)
    .decorations(false)
    .resizable(true)
    .always_on_top(true)
    .visible(true)
    .skip_taskbar(true)
    .build()
    .map_err(|e| format!("Failed to open AI window: {}", e))?;

    // Hide on close rather than destroy, so next open is instant
    let ai_window = ai_webview.as_ref().window().clone();
    ai_webview.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = ai_window.hide();
        }
    });

    Ok(())
}

fn toggle_window(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let window = window.as_ref().window();
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

// ---------------------------------------------------------
// TAURI ENTRYPOINT
// ---------------------------------------------------------
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = &*CONFIG;

    CLIPBOARD_MANAGER.start_monitoring();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            toggle_window(app);
        }))
        .setup(|app| {
            let app_handle = app.handle().clone();
            ipc_server::start_ipc_server(app_handle);

            let toggle = MenuItem::with_id(app, "toggle", "Toggle Window", true, None::<&str>)?;
            let quit   = MenuItem::with_id(app, "quit",   "Quit",          true, None::<&str>)?;
            let menu   = Menu::with_items(app, &[&toggle, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => toggle_window(app),
                    "quit"   => app.exit(0),
                    _        => {}
                })
                .build(app)?;

            let note_webview = tauri::WebviewWindowBuilder::new(
                app,
                "note",
                tauri::WebviewUrl::App("note".into()),
            )
            .title("quarry note")
            .inner_size(500.0, 400.0)
            .decorations(false)
            .resizable(true)
            .always_on_top(true)
            .visible(false)
            .skip_taskbar(true)
            .build()?;

            let note_window = note_webview.as_ref().window().clone();
            note_webview.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = note_window.hide();
                }
            });

            if let Some(webview) = app.get_webview_window("main") {
                let window = webview.as_ref().window().clone();
                webview.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            let _ = window.hide();
                        }
                        tauri::WindowEvent::Focused(true) => {
                            let _ = window.emit("window-focused", ());
                        }
                        _ => {}
                    }
                });

                #[cfg(target_os = "linux")]
                webview.with_webview(|wv| {
                    use webkit2gtk::WebViewExt;
                    use webkit2gtk::PermissionRequestExt;
                    let wk = wv.inner();
                    wk.connect_permission_request(|_, request| {
                        request.allow();
                        true
                    });
                })?;
            }

            match app.cli().matches() {
                Ok(matches) => {
                    if let Some(sub) = matches.subcommand {
                        if sub.name == "toggle" {}
                    }
                }
                Err(_) => {}
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            execute,
            clear_clipboard_history,
            get_theme,
            read_note,
            write_note,
            save_capture,
            get_groq_api_key,
            save_groq_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
