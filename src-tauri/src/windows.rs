use tauri::{Emitter, Manager};

pub fn setup_main_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let (w, h) = crate::CONFIG.read()
        .map(|c| (c.window.width as f64, c.window.height as f64))
        .unwrap_or((780.0, 520.0));

    let webview = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("".into()),
    )
    .title("quarry")
    .inner_size(w, h)
    .decorations(false)
    .transparent(true)
    .resizable(false)
    .always_on_top(true)
    .visible(false)
    .focused(false)
    .skip_taskbar(true)
    .build()?;

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

    Ok(())
}

pub fn setup_note_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let webview = tauri::WebviewWindowBuilder::new(
        app,
        "note",
        tauri::WebviewUrl::App("note".into()),
    )
    .title("quarry note")
    .inner_size(500.0, 400.0)
    .always_on_top(true)
    .visible(false)
    .build()?;

    let window = webview.as_ref().window().clone();
    webview.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window.hide();
        }
    });

    Ok(())
}

pub fn setup_settings_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let webview = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings".into()),
    )
    .title("quarry settings")
    .inner_size(780.0, 660.0)
    .min_inner_size(600.0, 700.0)
    .visible(false)
    .build()?;

    let window = webview.as_ref().window().clone();
    webview.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window.hide();
        }
    });

    Ok(())
}
