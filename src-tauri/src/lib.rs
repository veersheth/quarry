pub mod commands;
pub mod executor;
pub mod windows;
mod action_registry;
mod clipboard_manager;
mod config;
mod ipc_server;
pub mod pins;
mod searchers;
mod types;
mod usage_tracker;

use action_registry::ActionRegistry;
use clipboard_manager::ClipboardManager;
use usage_tracker::UsageHistory;

use commands::{
    cancel_rofi, clear_clipboard_history, exec_shell, execute, get_config, get_groq_api_key,
    get_theme, read_note, save_capture, save_config, save_groq_api_key, search, write_note,
};
use lazy_static::lazy_static;
use std::sync::{
    atomic::AtomicU64,
    RwLock,
};
use tauri::{
    Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

// ---------------------------------------------------------
// GLOBAL STATE
// ---------------------------------------------------------

pub(crate) static SEARCH_SEQ: AtomicU64 = AtomicU64::new(0);

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
    std::thread::spawn(|| { let _ = &*crate::commands::TRIGGERS; });
    crate::searchers::files::start_file_index();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // A second instance was launched — dispatch to the running app.
            match args.get(1).map(|s| s.as_str()) {
                Some("notepad")  => ipc_server::toggle_note_window(app),
                Some("settings") => ipc_server::toggle_settings_window(app),
                _                => ipc_server::toggle_window(app),
            }
        }))
        .setup(|app| {
            let app_handle = app.handle().clone();
            ipc_server::start_ipc_server(app_handle);

            let toggle   = MenuItem::with_id(app, "toggle",   "Toggle Launcher", true, None::<&str>)?;
            let note     = MenuItem::with_id(app, "note",     "Notepad",         true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings",        true, None::<&str>)?;
            let sep      = PredefinedMenuItem::separator(app)?;
            let quit     = MenuItem::with_id(app, "quit",     "Quit",            true, None::<&str>)?;
            let menu     = Menu::with_items(app, &[&toggle, &note, &settings, &sep, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle"   => ipc_server::toggle_window(app),
                    "note"     => {
                        if let Some(w) = app.get_webview_window("note") {
                            if w.is_visible().unwrap_or(false) { let _ = w.hide(); }
                            else { let _ = w.show(); let _ = w.set_focus(); }
                        }
                    }
                    "settings" => {
                        if let Some(w) = app.get_webview_window("settings") {
                            if w.is_visible().unwrap_or(false) { let _ = w.hide(); }
                            else { let _ = w.show(); let _ = w.set_focus(); }
                        }
                    }
                    "quit"     => app.exit(0),
                    _          => {}
                })
                .build(app)?;

            windows::setup_main_window(app)?;
            windows::setup_note_window(app)?;
            windows::setup_settings_window(app)?;

            // Handle CLI subcommands on first launch.
            use tauri_plugin_cli::CliExt;
            if let Ok(matches) = app.cli().matches() {
                if let Some(sub) = matches.subcommand {
                    match sub.name.as_str() {
                        "notepad"  => ipc_server::toggle_note_window(app.handle()),
                        "settings" => ipc_server::toggle_settings_window(app.handle()),
                        _          => {}
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            execute,
            exec_shell,
            clear_clipboard_history,
            get_theme,
            get_config,
            save_config,
            read_note,
            write_note,
            save_capture,
            get_groq_api_key,
            save_groq_api_key,
            cancel_rofi,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
