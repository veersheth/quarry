use tauri::{Emitter, Manager};

pub fn setup_note_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let note_webview = tauri::WebviewWindowBuilder::new(
        app,
        "note",
        tauri::WebviewUrl::App("note".into()),
    )
    .title("quarry note")
    .inner_size(500.0, 400.0)
    .decorations(false)
    .resizable(true)
    .always_on_top(true)
    .visible(false)
    .skip_taskbar(true)
    .build()?;

    let note_window = note_webview.as_ref().window().clone();
    note_webview.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = note_window.hide();
        }
    });

    Ok(())
}

pub fn setup_settings_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings_webview = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings".into()),
    )
    .title("quarry settings")
    .inner_size(780.0, 660.0)
    .min_inner_size(600.0, 700.0)
    .build()?;

    let win = settings_webview.as_ref().window().clone();
    settings_webview.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = win.hide();
        }
    });

    Ok(())
}

pub fn setup_main_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(webview) = app.get_webview_window("main") {
        let window = webview.as_ref().window().clone();
        webview.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                tauri::WindowEvent::Focused(true) => {
                    let _ = window.emit("window-focused", ());
                }
                _ => {}
            }
        });

        #[cfg(target_os = "linux")]
        webview.with_webview(|wv| {
            use webkit2gtk::PermissionRequestExt;
            use webkit2gtk::WebViewExt;
            let wk = wv.inner();
            wk.connect_permission_request(|_, request| {
                request.allow();
                true
            });
        })?;
    }

    Ok(())
}
