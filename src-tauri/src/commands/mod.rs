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
    let action_data = ACTION_REGISTRY
        .get_action(&action_id)
        .ok_or_else(|| format!("Action not found: {}", action_id))?;

    let tag = match &action_data {
        ActionData::CopyToClipboard { .. } | ActionData::CopyImageToClipboard { .. } => "copied",
        ActionData::RunFunction { function_name, .. } if function_name == "show_modal" => "stay",
        ActionData::RunFunction { function_name, .. } if function_name == "show_qr_clipboard" => {
            "stay"
        }
        ActionData::RunFunction { function_name, .. }
            if function_name == "copy_clipboard_image" =>
        {
            "copied"
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
    executor::execute_action(
        crate::types::ActionData::RunFunction {
            function_name: name,
            params,
        },
        &app,
    )
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

    Some(ClipboardFileMetadata {
        name,
        path,
        kind: if is_directory { "Folder" } else { "File" },
        extension,
        size_bytes,
        modified_unix,
    })
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
