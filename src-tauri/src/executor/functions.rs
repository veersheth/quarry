use std::sync::{atomic::AtomicBool, Arc};
use tauri::{Emitter, Manager};

use crate::searchers::bookmarks::BookmarksSearcher;

use super::{
    copy_image_to_clipboard, copy_to_clipboard, latest_in_dir, open_selecting, open_url,
    run_script_in_terminal, run_shell_command, spawn_timer_thread, toggle_pink_noise,
    toggle_rain_noise, ModalButton, ModalPayload,
};

pub(super) fn run_custom_function(
    function_name: &str,
    params: &[String],
    app: &tauri::AppHandle,
) -> Result<(), String> {
    match function_name {
        "run_in_terminal" => {
            let path = params.first().map(|s| s.as_str()).unwrap_or("");
            run_script_in_terminal(path)
        }
        // params: [raw_value, expr, formatted_display]
        "copy_calc" => {
            if params.is_empty() {
                return Err("copy_calc: missing params".into());
            }
            copy_to_clipboard(&params[0], app)?;
            if params.len() >= 3 {
                crate::searchers::math::push_to_history(&params[1], &params[2], &params[0]);
            }
            Ok(())
        }
        "open_note" => {
            if let Some(window) = app.get_webview_window("note") {
                let window = window.as_ref().window();
                let _ = window.show();
                let _ = window.set_focus();
            }
            Ok(())
        }
        "open_settings" => {
            if let Some(window) = app.get_webview_window("settings") {
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
            crate::PINS.add(
                &params[0],
                crate::pins::PinEntry {
                    name: params[1].clone(),
                    payload: params[2].clone(),
                },
            );
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
            });
            if let Some(win) = app.get_webview_window("main") {
                win.eval("location.reload()").ok();
            }
            Ok(())
        }
        "start_timer" => {
            let secs: u64 = params.first().and_then(|s| s.parse().ok()).unwrap_or(60);
            let label = params.get(1).cloned().unwrap_or_default();
            let id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let expires_at = crate::searchers::timer::now_secs() + secs;
            let cancelled = Arc::new(AtomicBool::new(false));
            crate::searchers::timer::add_timer(
                id,
                label.clone(),
                expires_at,
                secs,
                cancelled.clone(),
            );
            spawn_timer_thread(app.clone(), id, label, expires_at, secs, cancelled);
            Ok(())
        }
        "toggle_pink_noise" => {
            toggle_pink_noise();
            Ok(())
        }
        "toggle_rain_noise" => {
            toggle_rain_noise();
            Ok(())
        }
        "cancel_timer" => {
            let id: u64 = params.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            crate::searchers::timer::remove_timer(id);
            Ok(())
        }
        "run_shell" => {
            let command = params.first().ok_or("run_shell requires a command")?;
            run_shell_command(command)
        }
        "show_modal" => {
            static MODAL_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let seq = MODAL_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let body = params.first().cloned().unwrap_or_default();
            let buttons: Vec<ModalButton> = if params.len() > 1 {
                params[1..]
                    .iter()
                    .enumerate()
                    .map(|(i, spec)| {
                        let parts: Vec<&str> = spec.splitn(3, '|').collect();
                        let label = parts.first().copied().unwrap_or("OK").to_string();
                        let kind = parts
                            .get(1)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());
                        let action_id = parts.get(2).filter(|s| !s.is_empty()).map(|spec| {
                            let id = format!("modal_{}_{}", seq, i);
                            let data = if let Some(fn_spec) = spec.strip_prefix("fn:") {
                                // fn:function_name:arg1:arg2...
                                let mut it = fn_spec.splitn(2, ':');
                                let function_name = it.next().unwrap_or("").to_string();
                                let params = it
                                    .next()
                                    .map(|rest| rest.split(':').map(str::to_string).collect())
                                    .unwrap_or_default();
                                crate::types::ActionData::RunFunction {
                                    function_name,
                                    params,
                                }
                            } else {
                                crate::types::ActionData::RunFunction {
                                    function_name: "run_shell".to_string(),
                                    params: vec![spec.to_string()],
                                }
                            };
                            crate::ACTION_REGISTRY.register(id.clone(), data);
                            id
                        });
                        ModalButton {
                            label,
                            kind,
                            action_id,
                        }
                    })
                    .collect()
            } else {
                vec![ModalButton {
                    label: "Dismiss".into(),
                    kind: None,
                    action_id: None,
                }]
            };
            let payload = ModalPayload { body, buttons };
            app.emit("quarry-modal", &payload)
                .map_err(|e| e.to_string())
        }
        "copy_clipboard_image" => {
            let hash = params
                .first()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or("copy_clipboard_image requires a hash")?;
            let (full, width, height) = crate::CLIPBOARD_MANAGER
                .get_full_image(hash)
                .ok_or("Image not found in clipboard history")?;
            copy_image_to_clipboard(&full, width, height)
        }
        "pin_clipboard_image" => {
            let hash = params
                .first()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or("pin_clipboard_image requires a hash")?;
            let (full, thumbnail, width, height) = crate::CLIPBOARD_MANAGER
                .get_image_for_pin(hash)
                .ok_or("Image not found in clipboard history")?;
            let payload = serde_json::json!({
                "full": full,
                "thumbnail": thumbnail,
                "width": width,
                "height": height,
            })
            .to_string();
            crate::PINS.add(
                "clipboard_image",
                crate::pins::PinEntry {
                    name: hash.to_string(),
                    payload,
                },
            );
            Ok(())
        }
        "delete_clipboard_entry" => {
            let ts = params
                .first()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or("delete_clipboard_entry requires a timestamp")?;
            crate::CLIPBOARD_MANAGER.remove_by_timestamp(ts);
            Ok(())
        }
        "delete_file" => {
            if params.is_empty() {
                return Err("delete_file requires a path".into());
            }
            std::fs::remove_file(&params[0]).map_err(|e| e.to_string())
        }
        "trash_file" => {
            if params.is_empty() {
                return Err("trash_file requires a path".into());
            }
            trash::delete(&params[0]).map_err(|e| e.to_string())
        }
        "open_latest_download" => {
            let dir = dirs::download_dir().ok_or("Downloads directory not found")?;
            let newest = latest_in_dir(&dir).ok_or("Downloads folder is empty")?;
            open_url(&format!("file://{}", newest.to_string_lossy()), app)
        }
        "show_latest_download" => {
            let dir = dirs::download_dir().ok_or("Downloads directory not found")?;
            let newest = latest_in_dir(&dir).ok_or("Downloads folder is empty")?;
            open_selecting(&newest)
        }
        "open_latest_screenshot" => {
            let newest = [
                dirs::picture_dir().map(|p| p.join("Screenshots")),
                dirs::picture_dir(),
                dirs::home_dir().map(|p| p.join("Screenshots")),
            ]
            .into_iter()
            .flatten()
            .filter(|p| p.is_dir())
            .find_map(|dir| latest_in_dir(&dir))
            .ok_or("No screenshots found")?;
            open_url(&format!("file://{}", newest.to_string_lossy()), app)
        }
        "copy_date" => {
            let s = chrono::Local::now().format("%Y-%m-%d").to_string();
            copy_to_clipboard(&s, app)
        }
        "copy_time" => {
            let s = chrono::Local::now().format("%H:%M:%S").to_string();
            copy_to_clipboard(&s, app)
        }
        "copy_datetime" => {
            let s = chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%z")
                .to_string();
            copy_to_clipboard(&s, app)
        }
        "copy_local_ip" => {
            let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
            socket.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
            let ip = socket
                .local_addr()
                .map_err(|e| e.to_string())?
                .ip()
                .to_string();
            copy_to_clipboard(&ip, app)
        }
        "copy_hostname" => {
            let hostname = std::fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .or_else(|_| {
                    std::process::Command::new("hostname")
                        .output()
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                        .map_err(|e| e.to_string())
                })?;
            copy_to_clipboard(&hostname, app)
        }
        "copy_username" => {
            let user = std::env::var("USER")
                .or_else(|_| std::env::var("LOGNAME"))
                .map_err(|_| "USER environment variable not set".to_string())?;
            copy_to_clipboard(&user, app)
        }
        "ocr_screenshot" => {
            if params.is_empty() {
                return Err("ocr_screenshot requires a path".into());
            }
            let output = std::process::Command::new("tesseract")
                .args([&params[0], "stdout", "--psm", "3"])
                .output()
                .map_err(|e| format!("Failed to run tesseract: {}", e))?;
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() {
                return Err("No text found in screenshot".into());
            }
            copy_to_clipboard(&text, app)
        }
        "save_groq_api_key" => {
            let key = params.first().map(|s| s.as_str()).unwrap_or("");
            crate::config::Config::save_groq_api_key(key)
        }
        "show_qr_clipboard" => {
            // Find the most recent text entry in clipboard history and emit it
            // to the frontend to render as a QR code. No generation happens here —
            // all work is deferred to the frontend and only runs on trigger.
            let text = crate::CLIPBOARD_MANAGER.with_history(|history| {
                history.iter().find_map(|e| {
                    if let crate::clipboard_manager::ClipboardContent::Text { value } = &e.content {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
            });
            match text {
                Some(t) => app.emit("quarry-show-qr", t).map_err(|e| e.to_string()),
                None => Err("No text found in clipboard history".into()),
            }
        }
        "copy_last_image_ocr_text" => {
            let text = crate::CLIPBOARD_MANAGER.with_history(|history| {
                history.iter().find_map(|e| {
                    if let crate::clipboard_manager::ClipboardContent::Image {
                        ocr_text: Some(t),
                        ..
                    } = &e.content
                    {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
            });
            match text {
                Some(t) => copy_to_clipboard(&t, app),
                None => Err("No OCR text found on the last copied image".into()),
            }
        }
        _ => Err(format!("Unknown function: {}", function_name)),
    }
}
