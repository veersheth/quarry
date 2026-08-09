mod action_registry;
mod clipboard_manager;
pub mod commands;
mod config;
pub mod executor;
mod ipc_server;
pub mod pins;
mod search_utils;
mod searchers;
mod types;
mod usage_tracker;
pub mod windows;

use action_registry::ActionRegistry;
use clipboard_manager::ClipboardManager;
use usage_tracker::UsageHistory;

use commands::{
    cancel_rofi, clear_clipboard_history, exec_func, execute, get_ai_prefix,
    get_clipboard_thumbnail, get_config, get_groq_api_key, get_theme, read_note, save_capture,
    save_config, save_groq_api_key, search, start_drag, take_pending_query, write_note,
};
use lazy_static::lazy_static;
use std::sync::{atomic::AtomicU64, Mutex, RwLock};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

// ---------------------------------------------------------
// GLOBAL STATE
// ---------------------------------------------------------

pub(crate) static SEARCH_SEQ: AtomicU64 = AtomicU64::new(0);

// Query to pre-fill on first window focus after launch with `quarry <query>`
pub(crate) static PENDING_QUERY: Mutex<Option<String>> = Mutex::new(None);

lazy_static! {
    pub static ref CONFIG: RwLock<config::Config> = {
        config::Config::write_default_if_missing();
        RwLock::new(config::Config::load())
    };
    pub static ref USAGE_HISTORY: RwLock<UsageHistory> = RwLock::new(UsageHistory::load());
    pub(crate) static ref PINS: pins::PinStore = pins::PinStore::load();
    pub(crate) static ref ACTION_REGISTRY: ActionRegistry = ActionRegistry::new();
    pub(crate) static ref CLIPBOARD_MANAGER: ClipboardManager = {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("tauri");
        std::fs::create_dir_all(&data_dir).ok();
        let clipboard_path = data_dir.join("clipboard_history.json");
        ClipboardManager::with_storage(clipboard_path)
    };
}

// ---------------------------------------------------------
// TAURI ENTRYPOINT
// ---------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    drop(CONFIG.read());
    CLIPBOARD_MANAGER.start_monitoring();

    // Warm caches in background so the first search is instant
    std::thread::spawn(|| crate::searchers::apps::warm());
    std::thread::spawn(|| crate::searchers::emojis::warm());
    std::thread::spawn(|| crate::searchers::clipboard::warm());
    std::thread::spawn(|| {
        let _ = &*crate::commands::TRIGGERS;
    });
    crate::searchers::files::start_file_index();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // A second instance was launched - dispatch to the running app.
            let mut i = 1usize;
            let mut handled = false;
            while i < args.len() {
                match args[i].as_str() {
                    "--notepad" => { ipc_server::toggle_note_window(app); return; }
                    "--config"  => { ipc_server::toggle_settings_window(app); return; }
                    "--toggle"  => { ipc_server::toggle_window(app); return; }
                    "--with" => {
                        if let Some(q) = args.get(i + 1) {
                            ipc_server::show_with_query(app, q);
                            handled = true;
                            i += 1;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            if !handled {
                ipc_server::toggle_window(app);
            }
        }))
        .setup(|app| {
            let app_handle = app.handle().clone();
            ipc_server::start_ipc_server(app_handle);

            let toggle = MenuItem::with_id(app, "toggle", "Toggle Launcher", true, None::<&str>)?;
            let note = MenuItem::with_id(app, "note", "Notepad", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &note, &settings, &sep, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        ipc_server::toggle_window(tray.app_handle());
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => ipc_server::toggle_window(app),
                    "note" => {
                        if let Some(w) = app.get_webview_window("note") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                    "settings" => {
                        if let Some(w) = app.get_webview_window("settings") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            windows::inject_gtk_theme();
            windows::setup_main_window(app)?;
            windows::setup_note_window(app)?;
            windows::setup_settings_window(app)?;

            // Handle CLI flags on first launch.
            use tauri_plugin_cli::CliExt;
            if let Ok(matches) = app.cli().matches() {
                let args = &matches.args;
                if args.get("notepad").map(|a| a.occurrences > 0).unwrap_or(false) {
                    ipc_server::toggle_note_window(app.handle());
                } else if args.get("config").map(|a| a.occurrences > 0).unwrap_or(false) {
                    ipc_server::toggle_settings_window(app.handle());
                } else if let Some(q) = args.get("with").and_then(|a| a.value.as_str()) {
                    if let Ok(mut slot) = PENDING_QUERY.lock() {
                        *slot = Some(q.to_string());
                    }
                }
                // --toggle on first launch is a no-op: the window opens normally.
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            execute,
            clear_clipboard_history,
            get_clipboard_thumbnail,
            get_theme,
            get_config,
            get_ai_prefix,
            save_config,
            read_note,
            write_note,
            save_capture,
            get_groq_api_key,
            save_groq_api_key,
            cancel_rofi,
            start_drag,
            exec_func,
            take_pending_query,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
