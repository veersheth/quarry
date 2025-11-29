use base64;
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::borrow::Cow;

#[derive(serde::Serialize)]
struct ListItem {
    name: String,
    exec: Option<String>,
    description: Option<String>,
    icon: Option<String>,
}

fn clean_exec_field(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
fn get_apps() -> Vec<ListItem> {

    let mut apps = Vec::new();
    let locales = get_languages_from_env();

    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    for entry in entries {
        if entry.no_display() || entry.hidden() {
            continue;
        }

        let name = entry
            .name(&locales)
            .or_else(|| entry.name::<&str>(&[])) 
            .unwrap_or(Cow::Borrowed("Unknown"))
            .to_string();

        let exec = entry.exec().map(clean_exec_field);

        let description = entry
            .comment(&locales)
            .or_else(|| entry.comment::<&str>(&[]))
            .map(|s| s.to_string());

        let icon = None; // TODO

        apps.push(ListItem {
            name,
            exec,
            description,
            icon,
        });
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
