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
        ActionData::RunScript { path } => run_script(&path),
        ActionData::OpenInTerminal { path } => open_in_terminal(&path),
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

/// Run a script file in a visible terminal window so the user sees output.
fn run_script(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Use osascript to open Terminal and run the script
        let script = format!(
            r#"tell application "Terminal"
                activate
                do script "{}"
            end tell"#,
            path.replace('"', "\\\"")
        );
        Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .map_err(|e| format!("Failed to open Terminal: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let cmd_simple = format!("{path}; exec bash");
        let cmd_wrapped = format!("sh -c '{path}; exec bash'");
        let launched: &[(&str, &[&str])] = &[
            ("wezterm",        &["start", "--", "sh", "-c", &cmd_simple]),
            ("ghostty",        &["-e", "sh", "-c", &cmd_simple]),
            ("gnome-terminal", &["--", "sh", "-c", &cmd_simple]),
            ("kitty",          &["sh", "-c", &cmd_simple]),
            ("alacritty",      &["-e", "sh", "-c", &cmd_simple]),
            ("xfce4-terminal", &["-e", &cmd_wrapped]),
            ("xterm",          &["-e", &cmd_wrapped]),
        ];
        let mut ok = false;
        for (term, args) in launched {
            if Command::new(term).args(*args).spawn().is_ok() {
                ok = true;
                break;
            }
        }
        if !ok {
            return Err("No supported terminal emulator found".to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", path])
            .spawn()
            .map_err(|e| format!("Failed to open cmd: {}", e))?;
    }

    Ok(())
}

/// Open a directory in the system terminal.
fn open_in_terminal(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"tell application "Terminal"
                activate
                do script "cd {}"
            end tell"#,
            path.replace('"', "\\\"")
        );
        Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .map_err(|e| format!("Failed to open Terminal: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let launched = [
            ("gnome-terminal", vec!["--working-directory".to_string(), path.to_string()]),
            ("kitty",          vec!["--directory".to_string(),         path.to_string()]),
            ("alacritty",      vec!["--working-directory".to_string(), path.to_string()]),
            ("xfce4-terminal", vec!["--working-directory".to_string(), path.to_string()]),
            ("xterm",          vec!["-e".to_string(), format!("cd {path:?} && bash")]),
        ];
        let mut ok = false;
        for (term, args) in &launched {
            if Command::new(term).args(args).spawn().is_ok() {
                ok = true;
                break;
            }
        }
        if !ok {
            return Err("No supported terminal emulator found".to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try Windows Terminal first, fall back to cmd
        let wt = Command::new("wt").args(["-d", path]).spawn();
        if wt.is_err() {
            Command::new("cmd")
                .args(["/c", "start", "cmd"])
                .current_dir(path)
                .spawn()
                .map_err(|e| format!("Failed to open cmd: {}", e))?;
        }
    }

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
