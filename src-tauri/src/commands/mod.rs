mod search;
pub use search::{reload_triggers, search, warm_triggers};

use base64::{engine::general_purpose, Engine};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

use crate::config;
use crate::executor;
use crate::types::ActionData;
use crate::{ACTION_REGISTRY, CLIPBOARD_MANAGER, CONFIG, USAGE_HISTORY};

#[tauri::command]
pub fn execute(
    action_id: String,
    query: String,
    name: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let registered_action = ACTION_REGISTRY
        .get_action(&action_id)
        .ok_or_else(|| format!("Action not found: {}", action_id))?;

    if !matches!(registered_action, ActionData::ConfirmedAction { .. }) {
        if executor::request_confirmation(&app, &action_id, &registered_action)? {
            return Ok("stay".into());
        }
    }

    let confirmed = matches!(registered_action, ActionData::ConfirmedAction { .. });
    let action_data = match registered_action {
        ActionData::ConfirmedAction {
            action_id: original_id,
        } => ACTION_REGISTRY
            .get_action(&original_id)
            .ok_or_else(|| "The action to confirm is no longer available".to_string())?,
        action => action,
    };

    if let ActionData::SetQuery { query } = &action_data {
        if let Ok(mut history) = USAGE_HISTORY.write() {
            let display_name = name.as_deref().unwrap_or("Search");
            history.record_usage("searcher", &action_data.stable_id(), display_name);
        }
        return Ok(format!("set-query:{}", query));
    }

    let parent_item = ACTION_REGISTRY.get_parent_item(&action_id);
    let cache_parent = !matches!(
        &action_data,
        ActionData::CopyToClipboard { .. }
            | ActionData::PasteToApp { .. }
            | ActionData::CopyImageToClipboard { .. }
            | ActionData::CopyImageFile { .. }
    );

    let tag = match &action_data {
        ActionData::PasteToApp { .. } => "paste",
        ActionData::CopyToClipboard { .. } | ActionData::CopyImageToClipboard { .. } => "copied",
        ActionData::RunFunction { function_name, .. } if function_name == "show_modal" => "stay",
        ActionData::RunFunction { function_name, .. }
            if function_name == "open_setup_wizard"
                || function_name == "open_reset_confirmation"
                || function_name == "open_bookmark_creator"
                || function_name == "edit_bookmark"
                || function_name == "edit_searcher_trigger" =>
        {
            "stay"
        }
        ActionData::RunFunction { function_name, .. } if function_name == "show_qr_clipboard" => {
            "stay"
        }
        ActionData::RunFunction { function_name, .. }
            if function_name == "copy_clipboard_image" =>
        {
            "copied"
        }
        ActionData::RunFunction { function_name, .. }
            if function_name == "paste_clipboard_image" =>
        {
            "paste"
        }
        ActionData::RunFunction { function_name, .. } if function_name.starts_with("copy_") => {
            "copied"
        }
        ActionData::RunFunction { function_name, .. } if function_name == "trash_file" => {
            "toasted:Moved to trash"
        }
        ActionData::RunFunction { function_name, .. }
            if function_name == "pin" || function_name == "pin_clipboard_image" =>
        {
            "toasted:Pinned"
        }
        ActionData::RunFunction { function_name, .. } if function_name == "unpin" => {
            "toasted:Unpinned"
        }
        _ => "launched",
    };
    let tag = if confirmed { "refresh" } else { tag };

    let stable_id = action_data.stable_id();
    let result = executor::execute_action(action_data, &app);

    if result.is_ok() {
        if let Ok(mut history) = USAGE_HISTORY.write() {
            let display_name = name.as_deref().unwrap_or(&action_id);
            history.record_usage(&query, &stable_id, display_name);
        }
        if cache_parent {
            if let Some(item) = parent_item {
                ACTION_REGISTRY.cache_result_item(&item);
            }
        }
    }

    result.map(|_| tag.to_string())
}

/// Inject the paste shortcut into the application that was focused before Quarry.
/// The frontend calls this after hiding the launcher so the previous window regains focus.
#[tauri::command]
pub fn paste_to_focused_app() -> Result<(), String> {
    let terminal = active_window_is_terminal();
    let mut command = std::process::Command::new("wtype");
    if terminal {
        command.args([
            "-M", "ctrl", "-M", "shift", "v", "-m", "shift", "-m", "ctrl",
        ]);
    } else {
        command.args(["-M", "ctrl", "v", "-m", "ctrl"]);
    }

    let output = command.output().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "Paste typing helper `wtype` was not found. Install it or run Quarry from its Nix dev shell.".to_string()
        } else {
            format!("Failed to start wtype: {error}")
        }
    })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn active_window_is_terminal() -> bool {
    let Ok(output) = std::process::Command::new("hyprctl")
        .args(["-j", "activewindow"])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    let Ok(window) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return false;
    };
    let class = ["class", "initialClass"]
        .iter()
        .filter_map(|key| window.get(key).and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    const TERMINAL_CLASSES: &[&str] = &[
        "alacritty",
        "contour",
        "foot",
        "footclient",
        "ghostty",
        "gnome-terminal",
        "kitty",
        "konsole",
        "rio",
        "st-",
        "terminator",
        "tilix",
        "wezterm",
        "xterm",
        "urxvt",
        "xfce4-terminal",
    ];
    TERMINAL_CLASSES.iter().any(|name| class.contains(name)) || class.contains("terminal")
}

#[tauri::command]
pub fn exec_func(name: String, params: Vec<String>, app: tauri::AppHandle) -> Result<(), String> {
    let action = ActionData::RunFunction {
        function_name: name,
        params,
    };
    static EXEC_FUNC_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let action_id = format!(
        "exec_func_{}",
        EXEC_FUNC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    ACTION_REGISTRY.register(action_id.clone(), action.clone());
    if executor::request_confirmation(&app, &action_id, &action)? {
        return Ok(());
    }
    executor::execute_action(action, &app)
}

#[tauri::command]
pub fn take_pending_query() -> Option<String> {
    crate::PENDING_QUERY
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
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
pub fn get_sidebar_width(searcher: String) -> u32 {
    CONFIG
        .read()
        .ok()
        .and_then(|config| config.sidebar_widths.get(&searcher).copied())
        .unwrap_or(match searcher.as_str() {
            "files" => 280,
            _ => 224,
        })
}

#[tauri::command]
pub fn save_sidebar_width(searcher: String, width: u32) -> Result<(), String> {
    if !matches!(searcher.as_str(), "clipboard" | "files" | "fonts") {
        return Err(format!("Unknown sidebar searcher: {searcher}"));
    }

    let width = width.clamp(160, 640);
    let mut current = CONFIG.write().map_err(|error| error.to_string())?;
    let mut updated = current.clone();
    updated.sidebar_widths.insert(searcher, width);
    config::Config::save(&updated)?;
    *current = updated;
    Ok(())
}

#[tauri::command]
pub fn save_config(config: config::Config) -> Result<(), String> {
    let clipboard_tracking_enabled = config.clipboard_tracking_enabled;
    let file_indexing_enabled = config.file_indexing_enabled;
    config::Config::save(&config)?;
    if let Ok(mut c) = CONFIG.write() {
        *c = config;
    }
    reload_triggers();
    if clipboard_tracking_enabled {
        CLIPBOARD_MANAGER.start_monitoring();
    } else {
        CLIPBOARD_MANAGER.stop_monitoring();
    }
    if file_indexing_enabled {
        crate::searchers::files::start_file_index();
    } else {
        crate::searchers::files::stop_file_index();
    }
    Ok(())
}

#[tauri::command]
pub fn save_searcher_trigger(
    config_key: String,
    mode: String,
    value: String,
) -> Result<(), String> {
    let value = value.trim().to_string();
    let trigger_regex = match mode.as_str() {
        "prefix" => {
            if value.is_empty() || value.chars().any(char::is_whitespace) {
                return Err("Prefix must be a single, non-empty token".into());
            }
            format!(r"^{}\s+(.*)$", regex::escape(&value))
        }
        "regex" => {
            if value.is_empty() {
                return Err("Trigger regex cannot be empty".into());
            }
            value
        }
        _ => return Err("Unknown trigger editor mode".into()),
    };
    regex::Regex::new(&trigger_regex).map_err(|error| format!("Invalid trigger regex: {error}"))?;

    let mut current = CONFIG.write().map_err(|error| error.to_string())?;
    let mut config = current.clone();
    match config_key.as_str() {
        "apps" => config.triggers.apps = trigger_regex,
        "files" => config.triggers.files = trigger_regex,
        "clipboard" => config.triggers.clipboard = trigger_regex,
        "emojis" => config.triggers.emojis = trigger_regex,
        "fonts" => config.triggers.fonts = trigger_regex,
        "bookmarks" => config.triggers.bookmarks = trigger_regex,
        "system" => config.triggers.system = trigger_regex,
        "shortcuts" => config.triggers.shortcuts = trigger_regex,
        "scripts" => config.triggers.scripts = trigger_regex,
        "screenshots" => config.triggers.screenshots = trigger_regex,
        "math" => config.triggers.math = trigger_regex,
        "dictionary" => config.triggers.dictionary = trigger_regex,
        "time" => config.triggers.time = trigger_regex,
        "timer" => config.triggers.timer = trigger_regex,
        "ai" => config.triggers.ai = trigger_regex,
        "camera" => config.triggers.camera = trigger_regex,
        "color_picker" => config.triggers.color_picker = trigger_regex,
        "currency" => config.triggers.currency = trigger_regex,
        "qr_code" => config.triggers.qr_code = trigger_regex,
        "shell" => config.triggers.shell = trigger_regex,
        "lorem" => config.triggers.lorem = trigger_regex,
        key if key.starts_with("web:") => {
            let name = &key[4..];
            let web_search = config
                .web_searches
                .iter_mut()
                .find(|search| search.name == name)
                .ok_or_else(|| format!("Web searcher not found: {name}"))?;
            web_search.trigger = trigger_regex;
        }
        _ => return Err("Unknown searcher".into()),
    }

    config::Config::save(&config)?;
    *current = config;
    drop(current);
    reload_triggers();
    Ok(())
}

#[tauri::command]
pub fn get_clipboard_thumbnail(hash: String) -> Option<String> {
    let hash: u64 = hash.parse().ok()?;
    CLIPBOARD_MANAGER.get_thumbnail(hash)
}

#[tauri::command]
pub fn get_clipboard_image(hash: String) -> Option<String> {
    let hash: u64 = hash.parse().ok()?;
    CLIPBOARD_MANAGER
        .get_full_image(hash)
        .map(|(image, _, _)| image)
}

#[tauri::command]
pub fn get_action_text(action_id: String) -> Option<String> {
    match ACTION_REGISTRY.get_action(&action_id)? {
        ActionData::CopyToClipboard { text } => Some(text),
        _ => None,
    }
}

#[derive(serde::Serialize)]
pub struct ClipboardFileMetadata {
    name: String,
    path: String,
    kind: &'static str,
    extension: Option<String>,
    size_bytes: Option<u64>,
    modified_unix: Option<u64>,
    folder_child_count: Option<usize>,
    image_width: Option<u32>,
    image_height: Option<u32>,
}

#[tauri::command]
pub fn get_clipboard_file_metadata(path: String) -> Option<ClipboardFileMetadata> {
    let file_path = std::path::Path::new(&path);
    let metadata = std::fs::metadata(file_path).ok()?;
    let is_directory = metadata.is_dir();
    let name = file_path
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or(&path)
        .to_string();
    let extension = if is_directory {
        None
    } else {
        file_path
            .extension()
            .and_then(|part| part.to_str())
            .map(str::to_string)
    };
    let size_bytes = if is_directory {
        None
    } else {
        Some(metadata.len())
    };
    let modified_unix = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    let folder_child_count = if is_directory {
        std::fs::read_dir(file_path)
            .ok()
            .map(|children| children.take(101).filter(|entry| entry.is_ok()).count())
    } else {
        None
    };
    let is_image = matches!(
        extension.as_deref().map(str::to_ascii_lowercase).as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff")
    );
    let (image_width, image_height) = if is_image {
        image::image_dimensions(file_path)
            .ok()
            .map(|(width, height)| (Some(width), Some(height)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    Some(ClipboardFileMetadata {
        name,
        path,
        kind: if is_directory { "Folder" } else { "File" },
        extension,
        size_bytes,
        modified_unix,
        folder_child_count,
        image_width,
        image_height,
    })
}

#[tauri::command]
pub fn clear_clipboard_history(app: tauri::AppHandle) -> Result<(), String> {
    exec_func("clear_clipboard".into(), Vec::new(), app)
}

#[tauri::command]
pub fn create_bookmark(name: String, url: String) -> Result<(), String> {
    crate::searchers::bookmarks::BookmarksSearcher::add_bookmark(name, url).map(|_| ())
}

#[tauri::command]
pub fn update_bookmark(old_name: String, name: String, url: String) -> Result<(), String> {
    crate::searchers::bookmarks::BookmarksSearcher::update_bookmark(&old_name, name, url)
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
            let image = drag::Image::Raw(include_bytes!("../../../static/icons/file.png").to_vec());
            if let Ok(gtk_win) = win.gtk_window() {
                let _ = drag::start_drag(
                    &gtk_win,
                    item,
                    image,
                    move |result, _| match result {
                        drag::DragResult::Dropped => {
                            let _ = win_cb.hide();
                        }
                        _ => {
                            let _ = win_cb.set_always_on_top(true);
                        }
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
