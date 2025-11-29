mod searchers;
use searchers::apps::{AppSearcher, ListItem};
use searchers::emojis::EmojiSearcher;
use searchers::SearchProvider;

use std::process::Command;

#[tauri::command]
fn search(query: &str) -> Vec<ListItem> {
    if let Some(rest) = query.strip_prefix("em ") {
        return EmojiSearcher.search(rest);
    }

    AppSearcher.search(query)
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

