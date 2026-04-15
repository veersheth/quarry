use std::process::Command;

use base64::{engine::general_purpose, Engine};
use tauri::Manager;

use crate::searchers::bookmarks::BookmarksSearcher;
use crate::types::ActionData;

pub fn execute_action(action: ActionData, app: &tauri::AppHandle) -> Result<(), String> {
    match action {
        ActionData::LaunchApp { executable, args } => launch_app(&executable, &args),
        ActionData::OpenUrl { url } => open_url(&url, app),
        ActionData::CopyToClipboard { text } => copy_to_clipboard(&text, app),
        ActionData::CopyImageToClipboard { base64_png, width, height } => {
            copy_image_to_clipboard(&base64_png, width, height)
        }
        ActionData::RunFunction { function_name, params } => {
            run_custom_function(&function_name, &params, app)
        }
        ActionData::ShellCommand { command } => run_shell_command(&command),
        ActionData::None => Ok(()),
    }
}

fn launch_app(executable: &str, args: &[String]) -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    Command::new(executable)
        .args(args)
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", executable, e))?;
    Ok(())
}

fn open_url(url: &str, app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

fn copy_to_clipboard(text: &str, app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

fn copy_image_to_clipboard(base64_png: &str, width: u32, height: u32) -> Result<(), String> {
    let b64 = base64_png
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(base64_png);

    let png_bytes = general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("Failed to decode image base64: {}", e))?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to decode PNG: {}", e))?
        .into_rgba8();

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {}", e))?;

    let img_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: img.into_raw().into(),
    };

    clipboard
        .set_image(img_data)
        .map_err(|e| format!("Failed to set clipboard image: {}", e))?;

    Ok(())
}

fn run_shell_command(command: &str) -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to run shell command: {}", e))?;
    Ok(())
}

fn run_custom_function(
    function_name: &str,
    params: &[String],
    app: &tauri::AppHandle,
) -> Result<(), String> {
    match function_name {
        "open_note" => {
            if let Some(window) = app.get_webview_window("note") {
                let window = window.as_ref().window();
                let _ = window.show();
                let _ = window.set_focus();
            }
            Ok(())
        }
        "clear_clipboard" => {
            crate::CLIPBOARD_MANAGER.clear_history();
            Ok(())
        }
        "add_bookmark" => {
            if params.len() != 2 {
                return Err("add_bookmark requires name and url".to_string());
            }
            BookmarksSearcher::add_bookmark(params[0].clone(), params[1].clone()).map(|_| ())
        }
        "remove_bookmark" => {
            if params.is_empty() {
                return Err("remove_bookmark requires a name".to_string());
            }
            BookmarksSearcher::remove_bookmark(&params[0]).map(|_| ())
        }
        _ => Err(format!("Unknown function: {}", function_name)),
    }
}
