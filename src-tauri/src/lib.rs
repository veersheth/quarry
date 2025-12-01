mod searchers;
mod types;

use searchers::apps::AppSearcher;
use searchers::emojis::EmojiSearcher;
use searchers::math::MathSearcher;
use searchers::web_searchers::{GoogleSearcher, NixSearcher, URLSearcher, YouTubeSearcher};
use searchers::SearchProvider;
use tauri::Manager;
use types::SearchResult;

use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_cli::CliExt;

use crate::searchers::lorem::LoremSearcher;
use crate::searchers::processkiller::PkillSearcher;
use crate::searchers::shell::ShellSearcher;
use crate::searchers::web_searchers::GithubSearcher;

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
        (Regex::new(r"^gh\s+(.*)$").unwrap(), Box::new(GithubSearcher)),
        (Regex::new(r"^!\s+(.*)$").unwrap(), Box::new(ShellSearcher)),
        (Regex::new(r"^lorem\s+(.*)$").unwrap(), Box::new(LoremSearcher)),
        (Regex::new(r"^=\s+(.*)$").unwrap(), Box::new(MathSearcher)),
        (Regex::new(r"^kill\s+(.*)$").unwrap(), Box::new(PkillSearcher)),
        (Regex::new(r"^([0-9+\-*/^().\s]+)$").unwrap(), Box::new(MathSearcher)),
        (Regex::new(r"^app\s+(.*)$").unwrap(), Box::new(AppSearcher)),
    ];
}

// ---------------------------------------------------------
// SEARCH COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn search(query: &str, app: tauri::AppHandle) -> SearchResult {
    for (regex, searcher) in PREFIX_SEARCHERS.iter() {
        if let Some(caps) = regex.captures(query) {
            let rest = caps.get(1).map_or("", |m| m.as_str());
            return searcher.search(rest, &app);
        }
    }

    // fallback
    AppSearcher.search(query, &app)
}

// ---------------------------------------------------------
// EXECUTE COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn execute(executable: &str, app: tauri::AppHandle) -> Result<(), String> {
    let parts: Vec<&str> = executable.split_whitespace().collect();
    let executable = parts[0];
    let args = &parts[1..];

    Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", executable, e))?;

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
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, toggle the window of the first instance
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
                            // This will trigger the single_instance plugin
                            // which will toggle the window in the running instance
                            // and this instance will exit automatically
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
