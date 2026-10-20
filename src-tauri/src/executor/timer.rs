use std::sync::{atomic::AtomicBool, Arc};
use tauri::{Emitter, Manager};

use super::{audio, ModalButton, ModalPayload};

/// Spawn the background thread that waits until `expires_at` then fires the alert.
/// Works for both newly created timers and ones restored from a previous session.
pub fn spawn_timer_thread(
    app: tauri::AppHandle,
    id: u64,
    label: String,
    expires_at: u64,
    duration_secs: u64,
    cancelled: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let interval = std::time::Duration::from_millis(200);
        loop {
            std::thread::sleep(interval);
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            if crate::searchers::timer::now_secs() >= expires_at {
                break;
            }
        }
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        crate::searchers::timer::remove_timer(id);
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.show();
            let _ = win.set_focus();
        }
        let body = if label.is_empty() {
            format!(
                "{} timer has elapsed.",
                crate::searchers::timer::format_duration(duration_secs)
            )
        } else {
            format!(
                "**{}** \n\n{} timer elapsed.",
                label,
                crate::searchers::timer::format_duration(duration_secs)
            )
        };
        // Register a one-shot "start again" action for this specific timer.
        let restart_id = format!("timer_restart_{}", id);
        let mut restart_params = vec![duration_secs.to_string()];
        if !label.is_empty() {
            restart_params.push(label.clone());
        }
        crate::ACTION_REGISTRY.register(
            restart_id.clone(),
            crate::types::ActionData::RunFunction {
                function_name: "start_timer".into(),
                params: restart_params,
            },
        );
        let payload = ModalPayload {
            body,
            buttons: vec![
                ModalButton {
                    label: "Dismiss".into(),
                    kind: None,
                    action_id: None,
                },
                ModalButton {
                    label: format!(
                        "Start {} again",
                        crate::searchers::timer::format_duration(duration_secs)
                    ),
                    kind: Some("default".into()),
                    action_id: Some(restart_id),
                },
            ],
        };
        let _ = app.emit("quarry-modal", &payload);
        let sound = crate::CONFIG.read().unwrap().timer_sound.clone();
        std::thread::spawn(move || audio::play_timer_sound(&sound));
    });
}
