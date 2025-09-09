use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::process::Command;
use std::fs;
use base64;
use std::path::PathBuf;

#[derive(serde::Serialize)]
struct ListItem {
    name: String,
    exec: Option<String>,
    description: Option<String>,
    icon: Option<String>,
}

/// Attempts to resolve the icon name to an icon file path under /usr/share/icons or other fallback locations.
/// Returns the first found icon file path, or None if not found.
fn resolve_icon_path(icon_name: &str) -> Option<PathBuf> {
    // Usually icon names can be file names or simple names without extension.
    // Try these icon file extensions to find matching icon file.
    let icon_extensions = ["png", "svg", "xpm"];
    let icon_search_paths = vec![
        "/usr/share/icons/hicolor/48x48/apps", // common fallback path
        "/usr/share/pixmaps",                   // also common
    ];

    // If icon_name is an absolute path and exists, return it directly
    let icon_path = PathBuf::from(icon_name);
    if icon_path.is_absolute() && icon_path.exists() {
        return Some(icon_path);
    }

    // Try to find an icon file with appended extensions under search paths
    for ext in &icon_extensions {
        for base_path in &icon_search_paths {
            let candidate = PathBuf::from(base_path).join(format!("{}.{}", icon_name, ext));
            if candidate.exists() {
                return Some(candidate);
            }
        }
        // Also try icon_name itself if it has extension
        let candidate = PathBuf::from(format!("{}.{}", icon_name, ext));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Reads an icon file at specified path and returns a base64 data URI string ("data:image/...;base64,...") if successful.
fn icon_path_to_base64_data_uri(path: &PathBuf) -> Option<String> {
    if let Ok(icon_bytes) = fs::read(path) {
        // Detect mime type by extension for data URI prefix
        let mime_type = match path.extension().and_then(|e| e.to_str()) {
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            Some("xpm") => "image/x-xpixmap",
            _ => "application/octet-stream",
        };
        let base64_encoded = base64::encode(&icon_bytes);
        Some(format!("data:{};base64,{}", mime_type, base64_encoded))
    } else {
        None
    }
}

#[tauri::command]
fn get_apps() -> Vec<ListItem> {
    let mut apps = Vec::new();
    let locales = get_languages_from_env();
    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();
    for entry in entries {
        if let Some(name_cow) = entry.name(&locales) {
            let exec = entry.exec().map(|s| s.to_string());
            let description = entry.comment(&locales).map(|s| s.to_string());

            let icon = entry.icon().and_then(|icon_name| {
                // Try to resolve to file path
                resolve_icon_path(&icon_name)
                    .and_then(|path| icon_path_to_base64_data_uri(&path))
            });

            let app_entry = ListItem {
                name: name_cow.to_string(),
                exec,
                description,
                icon,
            };
            apps.push(app_entry);
        }
    }
    apps
}

#[tauri::command]
fn execute(executable: &str) -> Result<(), String> {
    let parts: Vec<&str> = executable.split_whitespace().collect();
    let executable = parts[0];
    let args = &parts[1..];
    Command::new(executable)
        .args(args)
        .spawn()
        // Command::new(executable)
        //     .spawn()
        .map_err(|e| format!("Failed to run {}: {}", executable, e))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_apps, execute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
