use std::process::Command;

use base64::{engine::general_purpose, Engine};
use serde::Serialize;
use tauri::{Emitter, Manager};

#[derive(Serialize, Clone)]
pub struct ModalButton {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ModalPayload {
    pub body: String,
    pub buttons: Vec<ModalButton>,
}

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
        ActionData::RofiSelect { name, response_socket } => rofi_select(&name, &response_socket),
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

    let spawn = |binary: &str, args: &[&str]| {
        Command::new(binary)
            .args(args)
            .process_group(0)
            .stdin(std::process::Stdio::null())
            .spawn()
            .is_ok()
    };

    // Respect $TERMINAL if set
    if let Ok(term) = std::env::var("TERMINAL") {
        if !term.is_empty() && spawn(&term, &["-e", path]) {
            return Ok(());
        }
    }

    // Modern freedesktop standard
    if spawn("xdg-terminal-exec", &[path]) {
        return Ok(());
    }

    // Known terminals
    if spawn("ghostty", &["-e", path])                    { return Ok(()); }
    if spawn("gnome-terminal", &["--", path])             { return Ok(()); }
    if spawn("kitty", &[path])                            { return Ok(()); }
    if spawn("alacritty", &["-e", path])                  { return Ok(()); }
    if spawn("wezterm", &["start", "--", path])           { return Ok(()); }
    if spawn("xfce4-terminal", &["-e", path])             { return Ok(()); }
    if spawn("xterm", &["-e", path])                      { return Ok(()); }

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
        "pin" => {
            if params.len() < 3 {
                return Err("pin requires searcher, name, payload".into());
            }
            crate::PINS.add(&params[0], crate::pins::PinEntry {
                name: params[1].clone(),
                payload: params[2].clone(),
            });
            Ok(())
        }
        "unpin" => {
            if params.len() < 2 {
                return Err("unpin requires searcher and name".into());
            }
            crate::PINS.remove(&params[0], &params[1]);
            Ok(())
        }
        "reload_quarry" => {
            if let Ok(mut cfg) = crate::CONFIG.write() {
                *cfg = crate::config::Config::load();
            }
            crate::commands::reload_triggers();
            crate::searchers::files::rebuild_index_now();
            std::thread::spawn(|| {
                crate::searchers::apps::reload();
                crate::searchers::settings::reload();
            });
            if let Some(win) = app.get_webview_window("main") {
                win.eval("location.reload()").ok();
            }
            Ok(())
        }
        "start_timer" => {
            let secs: u64 = params.first()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            let label = params.get(1).cloned().unwrap_or_default();
            let id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let expires_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + secs;
            let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            crate::searchers::timer::add_timer(id, label.clone(), expires_at, cancelled.clone());
            let app_clone = app.clone();
            std::thread::spawn(move || {
                let interval = std::time::Duration::from_millis(200);
                let total = std::time::Duration::from_secs(secs);
                let steps = (total.as_millis() / interval.as_millis()) as u64;
                for _ in 0..steps {
                    std::thread::sleep(interval);
                    if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                }
                if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                crate::searchers::timer::remove_timer(id);
                if let Some(win) = app_clone.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
                let body = if label.is_empty() {
                    format!("## Timer finished\nYour **{}** timer has expired.",
                        crate::searchers::timer::format_duration(secs))
                } else {
                    format!("## {}\nYour **{}** timer has expired.",
                        label, crate::searchers::timer::format_duration(secs))
                };
                let payload = ModalPayload {
                    body,
                    buttons: vec![ModalButton { label: "Dismiss".into(), kind: None, shell: None }],
                };
                let _ = app_clone.emit("quarry-modal", &payload);
                std::thread::spawn(play_beep);
            });
            Ok(())
        }
        "cancel_timer" => {
            let id: u64 = params.first()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            crate::searchers::timer::remove_timer(id);
            Ok(())
        }
        "show_modal" => {
            let body = params.first().cloned().unwrap_or_default();
            let buttons: Vec<ModalButton> = if params.len() > 1 {
                params[1..].iter().map(|spec| {
                    let parts: Vec<&str> = spec.splitn(3, '|').collect();
                    ModalButton {
                        label: parts.first().copied().unwrap_or("OK").to_string(),
                        kind:  parts.get(1).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                        shell: parts.get(2).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                    }
                }).collect()
            } else {
                vec![ModalButton { label: "Dismiss".into(), kind: None, shell: None }]
            };
            let payload = ModalPayload { body, buttons };
            app.emit("quarry-modal", &payload).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown function: {}", function_name)),
    }
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

fn play_beep() {
    use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
    use std::f32::consts::{FRAC_PI_2, TAU};

    let Ok((_stream, handle)) = OutputStream::try_default() else { return };
    let Ok(sink) = Sink::try_new(&handle) else { return };

    let sample_rate: u32 = 44100;
    let duration = 2.2f32;
    let n = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);

    let harmonics: &[(f32, f32)] = &[
        (130.8, 0.13), 
        (196.0, 0.09), 
        (261.6, 0.18), 
        (329.6, 0.13), 
        (392.0, 0.09), 
        (523.2, 0.07), 
    ];

    let attack  = 0.14f32; 
    let sustain = 0.25f32;
    let release = duration - attack - sustain;

    for i in 0..n {
        let t = i as f32 / sample_rate as f32;

        let env = if t < attack {
            ((t / attack) * FRAC_PI_2).sin()
        } else if t < attack + sustain {
            1.0
        } else {
            let r = (t - attack - sustain) / release;
            ((1.0 - r).max(0.0) * FRAC_PI_2).sin().powf(1.8)
        };

        let mut s = 0.0f32;
        for &(freq, amp) in harmonics {
            s += (TAU * freq * t).sin() * amp;
        }
        samples.push(s * env * 0.65);
    }

    let source = SamplesBuffer::new(1, sample_rate, samples);
    sink.append(source);
    sink.sleep_until_end();
}
