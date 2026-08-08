use tauri::{Emitter, Manager};
#[cfg(target_os = "linux")]
use gdk;

/// Inject a GTK CSS provider that themes the native CSD headerbar to match
/// quarry's dark aesthetic. Called once at startup before windows are created.
#[cfg(target_os = "linux")]
pub fn inject_gtk_theme() {
    use gtk::prelude::*;

    let css = r#"
        headerbar {
            background-color: #111113;
            background-image: none;
            border-bottom: 1px solid rgba(255,255,255,0.07);
            box-shadow: none;
            min-height: 36px;
            padding: 0 6px;
        }
        headerbar .title {
            color: rgba(255,255,255,0.28);
            font-size: 12px;
            font-weight: 500;
        }
        headerbar .subtitle { color: rgba(255,255,255,0.18); font-size: 11px; }
        headerbar button.titlebutton {
            background: transparent;
            background-image: none;
            border: 1px solid transparent;
            box-shadow: none;
            border-radius: 6px;
            padding: 4px;
            min-width: 0;
            min-height: 0;
            color: rgba(255,255,255,0.35);
        }
        headerbar button.titlebutton:hover {
            background-color: rgba(255,255,255,0.08);
            border-color: rgba(255,255,255,0.1);
            color: rgba(255,255,255,0.65);
        }
        headerbar button.titlebutton:active {
            background-color: rgba(255,255,255,0.14);
        }
        window.csd decoration,
        window.csd decoration:backdrop {
            box-shadow:
                0 0 0 1px rgba(255,255,255,0.09),
                0 8px 32px rgba(0,0,0,0.6);
            border-radius: 12px;
        }
    "#;

    let provider = gtk::CssProvider::new();
    if let Err(e) = provider.load_from_data(css.as_bytes()) {
        eprintln!("GTK CSS error: {e}");
        return;
    }
    if let Some(screen) = gdk::Screen::default() {
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

#[cfg(not(target_os = "linux"))]
pub fn inject_gtk_theme() {}

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

    // #1a1a1f - prevents white flash during resize/move
    #[cfg(target_os = "linux")]
    webview.with_webview(|wv| {
        use webkit2gtk::WebViewExt;
        if let Ok(color) = gdk::RGBA::parse("#1a1a1f") {
            wv.inner().set_background_color(&color);
        }
    })?;

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
    .min_inner_size(600.0, 400.0)
    .visible(false)
    .build()?;

    let window = webview.as_ref().window().clone();
    webview.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window.hide();
        }
    });

    // #111113 - prevents white flash during resize/move
    #[cfg(target_os = "linux")]
    webview.with_webview(|wv| {
        use webkit2gtk::WebViewExt;
        if let Ok(color) = gdk::RGBA::parse("#111113") {
            wv.inner().set_background_color(&color);
        }
    })?;

    Ok(())
}
