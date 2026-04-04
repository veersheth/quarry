mod action_registry;
mod clipboard_manager;
mod ipc_server;
mod searchers;
mod types;
mod usage_tracker;

use crate::searchers::camera::CameraSearcher;
use crate::searchers::bookmarks::BookmarksSearcher;
use crate::searchers::files::FileSearcher;
use crate::searchers::clipboard::ClipboardSearcher;
use crate::searchers::colorpicker::ColorPicker;
use crate::searchers::lorem::LoremSearcher;
use crate::searchers::shell::ShellSearcher;
use crate::searchers::system::SystemSearcher;
use searchers::apps::AppSearcher;
use searchers::dictionary::DictionarySearcher;
use searchers::emojis::EmojiSearcher;
use searchers::math::MathSearcher;
use searchers::web_searchers::{
    GitHubSearcher, GoogleSearcher, NixSearcher, URLSearcher, YouTubeSearcher,
};
use searchers::SearchProvider;

use action_registry::ActionRegistry;
use clipboard_manager::ClipboardManager;
use tauri::Manager;
use types::{ActionData, SearchResult};
use usage_tracker::{boost_results_by_usage, UsageHistory};

use base64::{engine::general_purpose, Engine};
use lazy_static::lazy_static;
use regex::Regex;
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
    static ref USAGE_HISTORY: RwLock<UsageHistory> = RwLock::new(UsageHistory::load());
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
// REGEX SEARCH DISPATCH TABLE
// ---------------------------------------------------------
lazy_static! {
    static ref PREFIX_SEARCHERS: Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> = vec![
        (Regex::new(r"^cam$").unwrap(), Box::new(CameraSearcher)),

        (Regex::new(r"^bk (.*)$").unwrap(), Box::new(BookmarksSearcher)),

        (Regex::new(r"^f\s+(.*)$").unwrap(), Box::new(FileSearcher)),

        (Regex::new(r"^cp\s+(.*)$").unwrap(), Box::new(ClipboardSearcher)),

        (Regex::new(r"^em\s+(.*)$").unwrap(), Box::new(EmojiSearcher)),

        (Regex::new(r"^(https?://\S+|(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(?:[:/]\S*)?)$").unwrap(), Box::new(URLSearcher)),

        (Regex::new(r"^g\s+(.*)$").unwrap(), Box::new(GoogleSearcher)),

        (Regex::new(r"^yt\s+(.*)$").unwrap(), Box::new(YouTubeSearcher)),

        (Regex::new(r"^nxp\s+(.*)$").unwrap(), Box::new(NixSearcher)),

        (Regex::new(r"^gh\s+(.*)$").unwrap(), Box::new(GitHubSearcher)),

        (Regex::new(r"^!\s+(.*)$").unwrap(), Box::new(ShellSearcher)),

        (Regex::new(r"^lorem\s+(.*)$").unwrap(), Box::new(LoremSearcher)),

        (Regex::new(r"^=\s+(.*)$").unwrap(), Box::new(MathSearcher)),

        (Regex::new(r"^def\s+(.*)$").unwrap(), Box::new(DictionarySearcher)),

        (Regex::new(r"^sys\s+(.*)$").unwrap(), Box::new(SystemSearcher)),

        (Regex::new(r"^color$").unwrap(), Box::new(ColorPicker)),

        (Regex::new(r"^app\s+(.*)$").unwrap(), Box::new(AppSearcher)),
    ];
}

// ---------------------------------------------------------
// SEARCH COMMAND
// ---------------------------------------------------------
#[tauri::command]
async fn search(query: String, app: tauri::AppHandle) -> Option<SearchResult> {
    let my_seq = SEARCH_SEQ.fetch_add(1, Ordering::SeqCst) + 1;

    let query_clone = query.clone(); 

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut result = None;

        for (regex, searcher) in PREFIX_SEARCHERS.iter() {
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
// EXECUTE COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn execute(action_id: String, query: String, app: tauri::AppHandle) -> Result<(), String> {
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
        ActionData::RunFunction {
            function_name,
            params,
        } => run_custom_function(&function_name, &params, &app),
        ActionData::ShellCommand { command } => run_shell_command(&command),
        ActionData::None => Ok(()),
    };

    if result.is_ok() {
        if let Ok(mut history) = USAGE_HISTORY.write() {
            history.record_usage(&query, &action_id, &action_id);
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
    // strip the data uri prefix if present
    let b64 = base64_png
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(base64_png);

    let png_bytes = general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("Failed to decode image base64: {}", e))?;

    // Decode PNG → raw RGBA bytes for arboard
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
    _app: &tauri::AppHandle,
) -> Result<(), String> {
    match function_name {
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
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => {
                        toggle_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            if let Some(webview) = app.get_webview_window("main") {
                let window = webview.as_ref().window().clone();
                webview.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window.hide();
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
                        if sub.name == "toggle" {
                            // trigger single_instance plugin
                        }
                    }
                }
                Err(_) => {}
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            execute,
            clear_clipboard_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
