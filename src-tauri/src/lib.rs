mod searchers;
mod types;
mod usage_tracker;
mod action_registry;

use searchers::apps::AppSearcher;
use searchers::emojis::EmojiSearcher;
use searchers::math::MathSearcher;
use searchers::web_searchers::{GoogleSearcher, NixSearcher, URLSearcher, YouTubeSearcher, GitHubSearcher};
use crate::searchers::lorem::LoremSearcher;
use crate::searchers::shell::ShellSearcher;
use searchers::SearchProvider;

use tauri::Manager;
use types::{SearchResult, ActionData};
use usage_tracker::{UsageHistory, boost_results_by_usage};
use action_registry::ActionRegistry;

use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_cli::CliExt;


// ---------------------------------------------------------
// GLOBAL STATE
// ---------------------------------------------------------
lazy_static! {
    static ref USAGE_HISTORY: Mutex<UsageHistory> = Mutex::new(UsageHistory::load());
    static ref ACTION_REGISTRY: Mutex<ActionRegistry> = Mutex::new(ActionRegistry::new());
}

// ---------------------------------------------------------
// REGEX SEARCH DISPATCH TABLE
// ---------------------------------------------------------
lazy_static! {
    static ref PREFIX_SEARCHERS: Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> = vec![
        (Regex::new(r"^em\s+(.*)$").unwrap(), Box::new(EmojiSearcher)),
        (Regex::new(r"^(https?://.*)$").unwrap(), Box::new(URLSearcher)),
        (Regex::new(r"^g\s+(.*)$").unwrap(), Box::new(GoogleSearcher)),
        (Regex::new(r"^yt\s+(.*)$").unwrap(), Box::new(YouTubeSearcher)),
        (Regex::new(r"^nxp\s+(.*)$").unwrap(), Box::new(NixSearcher)),
        (Regex::new(r"^gh\s+(.*)$").unwrap(), Box::new(GitHubSearcher)),
        (Regex::new(r"^!\s+(.*)$").unwrap(), Box::new(ShellSearcher)),
        (Regex::new(r"^lorem\s+(.*)$").unwrap(), Box::new(LoremSearcher)),
        (Regex::new(r"^=\s+(.*)$").unwrap(), Box::new(MathSearcher)),
        (Regex::new(r"^([0-9+\-*/^().\s]+)$").unwrap(), Box::new(MathSearcher)),
        (Regex::new(r"^app\s+(.*)$").unwrap(), Box::new(AppSearcher)),
    ];
}

// ---------------------------------------------------------
// SEARCH COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn search(query: &str, app: tauri::AppHandle) -> SearchResult {
    let mut result = None;
    
    for (regex, searcher) in PREFIX_SEARCHERS.iter() {
        if let Some(caps) = regex.captures(query) {
            let rest = caps.get(1).map_or("", |m| m.as_str());
            result = Some(searcher.search(rest, &app));
            break;
        }
    }

    let mut search_result = result.unwrap_or_else(|| AppSearcher.search(query, &app));

    // Boost results based on usage history
    if let Ok(history) = USAGE_HISTORY.lock() {
        search_result.results = boost_results_by_usage(
            search_result.results,
            query,
            &history,
        );
    }

    search_result
}

// ---------------------------------------------------------
// EXECUTE COMMAND 
// ---------------------------------------------------------
#[tauri::command]
fn execute(action_id: String, query: String, app: tauri::AppHandle) -> Result<(), String> {
    let action_data = ACTION_REGISTRY
        .lock()
        .map_err(|e| format!("Failed to lock registry: {}", e))?
        .get_action(&action_id)
        .ok_or_else(|| format!("Action not found: {}", action_id))?;

    let result = match action_data {
        ActionData::LaunchApp { executable, args } => {
            launch_app(&executable, &args)
        }
        ActionData::OpenUrl { url } => {
            open_url(&url, &app)
        }
        ActionData::CopyToClipboard { text } => {
            copy_to_clipboard(&text, &app)
        }
        ActionData::RunFunction { function_name, params } => {
            run_custom_function(&function_name, &params, &app)
        }
        ActionData::ShellCommand { command } => {
            run_shell_command(&command)
        }
    };

    // record usage if execution successful
    if result.is_ok() {
        if let Ok(mut history) = USAGE_HISTORY.lock() {
            history.record_usage(&query, &action_id, &action_id);
        }
    }

    result
}

// ---------------------------------------------------------
// EXECUTION HANDLERS
// ---------------------------------------------------------
fn launch_app(executable: &str, args: &[String]) -> Result<(), String> {
    Command::new(executable)
        .args(args)
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

fn run_shell_command(command: &str) -> Result<(), String> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .map_err(|e| format!("Failed to run shell command: {}", e))?;
    Ok(())
}

fn run_custom_function(function_name: &str, params: &[String], app: &tauri::AppHandle) -> Result<(), String> {
    match function_name {
        "example_function" => {
            println!("Running example function with params: {:?}", params);
            Ok(())
        }
        _ => Err(format!("Unknown function: {}", function_name))
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
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            toggle_window(app);
        }))
        .setup(|app| {
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
            }

            match app.cli().matches() {
                Ok(matches) => {
                    if let Some(sub) = matches.subcommand {
                        if sub.name == "toggle" {
                            // Trigger single_instance plugin
                        }
                    }
                }
                Err(_) => {}
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![search, execute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

