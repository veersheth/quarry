use std::path::PathBuf;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(serde::Deserialize, serde::Serialize)]
enum IpcCommand {
    Toggle,
    Show,
    Hide,
    Ping,
    ToggleNote,
}

#[derive(serde::Serialize)]
struct IpcResponse {
    success: bool,
    message: String,
}

fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .unwrap_or_else(|_| "/tmp".to_string());
    
    PathBuf::from(runtime_dir).join("quarry.sock")
}

pub fn start_ipc_server(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let socket_path = get_socket_path();
        
        let _ = std::fs::remove_file(&socket_path); // clean up old socket
        
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind IPC socket: {}", e);
                return;
            }
        };
        
        // set socket permissions, unix only
        #[cfg(unix)]
        {
            if let Ok(metadata) = std::fs::metadata(&socket_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o600); // Only owner can read/write
                let _ = std::fs::set_permissions(&socket_path, permissions);
            }
        }
        
        println!("IPC server listening on: {}", socket_path.display());
        
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let app = app_handle.clone();
                    tauri::async_runtime::spawn(handle_client(stream, app));
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    });
}

async fn handle_client(stream: UnixStream, app_handle: tauri::AppHandle) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    match reader.read_line(&mut line).await {
        Ok(0) => return, // closed connectaion
        Ok(_) => {
            let response = match serde_json::from_str::<IpcCommand>(line.trim()) {
                Ok(cmd) => handle_command(cmd, &app_handle),
                Err(e) => IpcResponse {
                    success: false,
                    message: format!("Invalid command: {}", e),
                },
            };
            
            let response_json = serde_json::to_string(&response).unwrap() + "\n";
            let _ = writer.write_all(response_json.as_bytes()).await;
        }
        Err(e) => {
            eprintln!("Error reading from client: {}", e);
        }
    }
}

// factory IPC command handler
fn handle_command(cmd: IpcCommand, app_handle: &tauri::AppHandle) -> IpcResponse {
    match cmd {
        IpcCommand::Toggle => {
            toggle_window(app_handle);
            IpcResponse {
                success: true,
                message: "Window toggled".to_string(),
            }
        }
        IpcCommand::Show => {
            show_window(app_handle);
            IpcResponse {
                success: true,
                message: "Window shown".to_string(),
            }
        }
        IpcCommand::Hide => {
            hide_window(app_handle);
            IpcResponse {
                success: true,
                message: "Window hidden".to_string(),
            }
        }
        IpcCommand::Ping => IpcResponse {
            success: true,
            message: "pong".to_string(),
        },
        IpcCommand::ToggleNote => {
            toggle_note_window(app_handle);
            IpcResponse {
                success: true,
                message: "Note window toggled".to_string(),
            }
        }
    }
}

pub(crate) fn toggle_window(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let window = window.as_ref().window();
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn show_window(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let window = window.as_ref().window();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_window(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let window = window.as_ref().window();
        let _ = window.hide();
    }
}

fn toggle_note_window(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("note") {
        let window = window.as_ref().window();
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
