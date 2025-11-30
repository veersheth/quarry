mod searchers;

use searchers::apps::{AppSearcher, ListItem};
use searchers::emojis::EmojiSearcher;
use searchers::math::MathSearcher;
use searchers::web_searchers::{GoogleSearcher, NixSearcher, URLSearcher, YouTubeSearcher};
use searchers::SearchProvider;

use std::process::Command;
use lazy_static::lazy_static;
use regex::Regex;

// ---------------------------------------------------------
// REGEX SEARCH DISPATCH TABLE
// ---------------------------------------------------------
lazy_static! {
    static ref PREFIX_SEARCHERS:
        Vec<(Regex, Box<dyn SearchProvider + Send + Sync>)> = vec![

        (Regex::new(r"^em\s+(.*)$").unwrap(), Box::new(EmojiSearcher)),
        (Regex::new(r"^(https?://.*)$").unwrap(), Box::new(URLSearcher)),
        (Regex::new(r"^g\s+(.*)$").unwrap(), Box::new(GoogleSearcher)),
        (Regex::new(r"^yt\s+(.*)$").unwrap(), Box::new(YouTubeSearcher)),
        (Regex::new(r"^nxp\s+(.*)$").unwrap(), Box::new(NixSearcher)),
        (Regex::new(r"^=\s+(.*)$").unwrap(), Box::new(MathSearcher)),
        (Regex::new(r"^app\s+(.*)$").unwrap(), Box::new(AppSearcher)),

    ];
}

// ---------------------------------------------------------
// SEARCH COMMAND
// ---------------------------------------------------------
#[tauri::command]
fn search(query: &str) -> Vec<ListItem> {

    for (regex, searcher) in PREFIX_SEARCHERS.iter() {
        if let Some(caps) = regex.captures(query) {
            let rest = caps.get(1).map_or("", |m| m.as_str());
            return searcher.search(rest);
        }
    }

    // fallback
    AppSearcher.search(query)
}

// ---------------------------------------------------------
// EXECUTE COMMAND
// ---------------------------------------------------------
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

// ---------------------------------------------------------
// TAURI ENTRYPOINT
// ---------------------------------------------------------
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![search, execute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

