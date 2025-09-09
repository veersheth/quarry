use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::process::Command;

#[derive(serde::Serialize)]
struct ListItem {
    name: String,
    exec: Option<String>,
    description: Option<String>,
    icon: Option<String>,
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
            let icon = entry.icon().map(|s| s.to_string());

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
