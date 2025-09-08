use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(serde::Serialize)]
struct ListItem {
    name: String,
    executable: String,
}

#[tauri::command]
fn get_apps() -> Vec<ListItem> {
    let mut apps = Vec::new();

    // directories containing .desktop entries
    let dirs = vec![
        PathBuf::from("/usr/share/applications"),
        dirs::home_dir().unwrap().join(".local/share/applications"),
    ];

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let mut name = String::new();
                    let mut exec = String::new();

                    for line in content.lines() {
                        if let Some(rest) = line.strip_prefix("Name=") {
                            name = rest.to_string();
                        } else if let Some(rest) = line.strip_prefix("Exec=") {
                            exec = rest.to_string();
                        }
                    }

                    if !name.is_empty() && !exec.is_empty() {
                        apps.push(ListItem {
                            name,
                            executable: exec,
                        });
                    }
                }
            }
        }
    }

    apps
}

#[tauri::command]
fn run_app(executable: &str) -> Result<(), String> {
    Command::new(executable)
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", executable, e))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_apps, run_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
