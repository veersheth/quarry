use base64::{engine::general_purpose, Engine};
use serde::Serialize;
use std::process::Command;

mod audio;
mod functions;
mod timer;
pub(super) use audio::{toggle_pink_noise, toggle_rain_noise};
pub(crate) use audio::{PINK_NOISE_RUNNING, RAIN_NOISE_RUNNING};
pub(crate) use timer::spawn_timer_thread;

#[derive(Serialize, Clone)]
struct ModalButton {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
}

#[derive(Serialize, Clone)]
struct ModalPayload {
    pub body: String,
    pub buttons: Vec<ModalButton>,
}

use crate::types::ActionData;

pub(crate) fn execute_action(action: ActionData, app: &tauri::AppHandle) -> Result<(), String> {
    match action {
        ActionData::LaunchApp { executable, args } => launch_app(&executable, &args),
        ActionData::OpenUrl { url } => open_url(&url, app),
        ActionData::CopyToClipboard { text } => copy_to_clipboard(&text, app),
        ActionData::PasteToApp { text } => copy_to_clipboard(&text, app),
        ActionData::CopyImageToClipboard {
            base64_png,
            width,
            height,
        } => copy_image_to_clipboard(&base64_png, width, height),
        ActionData::CopyImageFile { path } => copy_image_file_to_clipboard(&path),
        ActionData::RunFunction {
            function_name,
            params,
        } => functions::run_custom_function(&function_name, &params, app),
        ActionData::ShellCommand { command } => run_shell_command(&command),
        ActionData::RunScript { path } => run_script(&path),
        ActionData::OpenInTerminal { path } => open_in_terminal(&path),
        ActionData::RofiSelect {
            name,
            response_socket,
        } => rofi_select(&name, &response_socket),
        ActionData::SetQuery { .. } => {
            Err("Search query actions are handled by the frontend".into())
        }
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

fn copy_image_file_to_clipboard(path: &str) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .into_rgba8();
    let (width, height) = img.dimensions();
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .set_image(arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: img.into_raw().into(),
        })
        .map_err(|e| format!("Failed to set clipboard image: {}", e))
}

pub(crate) fn run_shell_command(command: &str) -> Result<(), String> {
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

fn run_script(path: &str) -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    Command::new("sh")
        .args(["-c", path])
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn run_script_in_terminal(path: &str) -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    let spawn = |binary: &str, args: &[&str]| {
        Command::new(binary)
            .args(args)
            .process_group(0)
            .stdin(std::process::Stdio::null())
            .spawn()
            .is_ok()
    };

    if let Ok(term) = std::env::var("TERMINAL") {
        if !term.is_empty() && spawn(&term, &["-e", path]) {
            return Ok(());
        }
    }

    if spawn("xdg-terminal-exec", &[path]) {
        return Ok(());
    }
    if spawn("ghostty", &["-e", path]) {
        return Ok(());
    }
    if spawn("gnome-terminal", &["--", path]) {
        return Ok(());
    }
    if spawn("kitty", &[path]) {
        return Ok(());
    }
    if spawn("alacritty", &["-e", path]) {
        return Ok(());
    }
    if spawn("wezterm", &["start", "--", path]) {
        return Ok(());
    }
    if spawn("xfce4-terminal", &["-e", path]) {
        return Ok(());
    }
    if spawn("xterm", &["-e", path]) {
        return Ok(());
    }

    Err("No terminal found. Set the $TERMINAL environment variable.".to_string())
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
        use std::os::unix::process::CommandExt;
        let spawn = |binary: &str, args: &[&str]| {
            Command::new(binary)
                .args(args)
                .process_group(0)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok()
        };

        // POSIX-safe single-quote the path for the cd fallback command
        let quoted = format!("'{}'", path.replace('\'', "'\\''"));
        let cd_cmd = format!("cd {} && exec ${{SHELL:-bash}}", quoted);

        // Respect $TERMINAL first (consistent with run_script)
        if let Ok(term) = std::env::var("TERMINAL") {
            if !term.is_empty() && spawn(&term, &["-e", "sh", "-c", &cd_cmd]) {
                return Ok(());
            }
        }

        // Modern freedesktop standard
        if spawn("xdg-terminal-exec", &["--working-directory", path]) {
            return Ok(());
        }

        // Known terminals with native working-directory support
        if spawn("ghostty", &[&format!("--working-directory={}", path)]) {
            return Ok(());
        }
        if spawn("wezterm", &["start", "--cwd", path]) {
            return Ok(());
        }
        if spawn("gnome-terminal", &["--working-directory", path]) {
            return Ok(());
        }
        if spawn("kitty", &["--directory", path]) {
            return Ok(());
        }
        if spawn("alacritty", &["--working-directory", path]) {
            return Ok(());
        }
        if spawn("xfce4-terminal", &["--working-directory", path]) {
            return Ok(());
        }
        if spawn("xterm", &["-e", "sh", "-c", &cd_cmd]) {
            return Ok(());
        }

        return Err("No terminal found. Set the $TERMINAL environment variable.".to_string());
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
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

/// Open a file manager with `path` selected/highlighted.
/// Detects the default file manager via xdg-mime, then $FILE_MANAGER,
/// then tries known managers, finally falls back to opening the parent folder.
fn open_selecting(path: &std::path::Path) -> Result<(), String> {
    let path_str = path.to_string_lossy().to_string();
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());

    // Detect the default file manager registered for directories
    let desktop = Command::new("xdg-mime")
        .args(["query", "default", "inode/directory"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();

    let spawned = if desktop.contains("nautilus") {
        Command::new("nautilus")
            .args(["--select", &path_str])
            .spawn()
            .is_ok()
    } else if desktop.contains("dolphin") {
        Command::new("dolphin")
            .args(["--select", &path_str])
            .spawn()
            .is_ok()
    } else if desktop.contains("nemo") {
        Command::new("nemo")
            .args(["--no-desktop", &path_str])
            .spawn()
            .is_ok()
    } else if desktop.contains("thunar") {
        Command::new("thunar").arg(&parent).spawn().is_ok()
    } else if desktop.contains("pcmanfm") {
        Command::new("pcmanfm").arg(&parent).spawn().is_ok()
    } else if let Ok(fm) = std::env::var("FILE_MANAGER") {
        // User-defined override
        Command::new(&fm).arg(&path_str).spawn().is_ok()
    } else {
        // No detection - try common managers in order
        Command::new("nautilus")
            .args(["--select", &path_str])
            .spawn()
            .is_ok()
            || Command::new("dolphin")
                .args(["--select", &path_str])
                .spawn()
                .is_ok()
            || Command::new("nemo")
                .args(["--no-desktop", &path_str])
                .spawn()
                .is_ok()
    };

    if spawned {
        return Ok(());
    }

    // Final fallback: open the parent directory
    Command::new("xdg-open")
        .arg(&parent)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn latest_in_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .max_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })
        .map(|e| e.path())
}

fn rofi_select(name: &str, response_socket: &str) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    let mut stream = UnixStream::connect(response_socket)
        .map_err(|e| format!("Failed to connect to rofi socket: {}", e))?;
    stream
        .write_all(format!("{}\n", name).as_bytes())
        .map_err(|e| format!("Failed to write to rofi socket: {}", e))?;
    Ok(())
}
