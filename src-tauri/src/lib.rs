mod searchers;
use searchers::apps::{self, ListItem};

use std::process::Command;

#[tauri::command]
fn search(query: &str) -> Vec<ListItem> {
    let apps_list = apps::get_apps();

    let query_lower = query.to_lowercase();

    apps_list
        .into_iter()
        .filter(|item| {
            item.name.to_lowercase().contains(&query_lower)
                || item
                    .description
                    .as_ref()
                    .map_or(false, |d| d.to_lowercase().contains(&query_lower))
                || item
                    .exec
                    .as_ref()
                    .map_or(false, |e| e.to_lowercase().contains(&query_lower))
        })
        .collect()
}

#[tauri::command]
fn execute(executable: &str) -> Result<(), String> {
    let parts: Vec<&str> = executable.split_whitespace().collect();
    let executable = parts[0];
    let args = &parts[1..];

    Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", executable, e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![search, execute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
