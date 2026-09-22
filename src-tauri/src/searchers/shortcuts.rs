use tauri::AppHandle;
use super::{SearchProvider, ICON_FOLDER, ICON_FILE, ICON_SETTINGS, ICON_TRANSPARENT};
use crate::types::{Action, ActionData, ResultItem, SearchResult};

pub struct ShortcutsSearcher;

fn item(name: &str, description: &str, icon: &str, action: ActionData) -> ResultItem {
    ResultItem::new(name, vec![Action::new("Run", action)])
        .description(description)
        .icon(icon)
}

fn folder_item(name: &str, description: &str, path: std::path::PathBuf) -> ResultItem {
    let url = format!("file://{}", path.to_string_lossy());
    item(name, description, ICON_FOLDER, ActionData::OpenUrl { url })
}

fn shell(cmd: &'static str) -> ActionData {
    ActionData::ShellCommand { command: cmd.into() }
}

fn func(name: &'static str, params: Vec<String>) -> ActionData {
    ActionData::RunFunction { function_name: name.into(), params }
}

fn all_shortcuts() -> Vec<ResultItem> {
    let mut items: Vec<ResultItem> = Vec::new();

    // ── Quarry ────────────────────────────────────────────────────────────
    items.push(item("Reload Quarry", "Rebuild file index, refresh app list, reload styles",
        ICON_TRANSPARENT, func("reload_quarry", vec![])));
    items.push(item("Open Quarry Settings", "Open the quarry settings window",
        ICON_TRANSPARENT, func("open_settings", vec![])));

    if let Some(cfg) = dirs::config_dir() {
        items.push(folder_item(
            "Open Quarry Config Folder",
            "Open ~/.config/quarry in your file manager",
            cfg.join("quarry"),
        ).icon(ICON_TRANSPARENT));

        if let Some(sp) = super::scripts::scripts_dir() {
            let desc = format!("Open {} — place executable scripts here", sp.to_string_lossy());
            items.push(folder_item("Open Scripts Folder", &desc, sp));
        }
    }

    // ── Trash ─────────────────────────────────────────────────────────────
    items.push(item("Open Trash", "Open the trash folder - deleted files recycle bin",
        ICON_FOLDER, shell("gio open trash://")));
    items.push(item("Empty Trash", "Permanently delete all trashed files - clear recycle bin",
        ICON_FOLDER, ActionData::modal(
            "## Empty Trash\n\nThis will permanently delete all trashed files.",
            &[("Empty Trash", "danger", "gio trash --empty"), ("Cancel", "default", "")],
        )));

    // ── Files ─────────────────────────────────────────────────────────────
    items.push(item("Open Latest Download", "Open the most recently downloaded file",
        ICON_FILE, func("open_latest_download", vec![])));
    items.push(item("Show Latest Download", "Open the Downloads folder with the latest file selected",
        ICON_FOLDER, func("show_latest_download", vec![])));
    items.push(item("Open Latest Screenshot", "Open the most recently taken screenshot",
        ICON_FILE, func("open_latest_screenshot", vec![])));

    // ── Folders ───────────────────────────────────────────────────────────
    if let Some(p) = dirs::download_dir() { items.push(folder_item("Open Downloads", "Open the Downloads folder", p)); }
    if let Some(p) = dirs::home_dir()     { items.push(folder_item("Open Home Folder", "Open your home directory", p)); }
    if let Some(p) = dirs::picture_dir()  { items.push(folder_item("Open Pictures", "Open the Pictures folder", p)); }
    if let Some(p) = dirs::document_dir() { items.push(folder_item("Open Documents", "Open the Documents folder", p)); }
    if let Some(p) = dirs::desktop_dir()  { items.push(folder_item("Open Desktop", "Open the Desktop folder", p)); }

    // ── Copy to clipboard ─────────────────────────────────────────────────
    items.push(item("Convert Last Copied Text to QR Code",
        "Generate a QR code from the most recently copied text",
        ICON_TRANSPARENT, func("show_qr_clipboard", vec![])));
    items.push(item("Copy Text from Last Copied Image",
        "Extract and copy OCR text from the most recently copied image",
        ICON_FILE, func("copy_last_image_ocr_text", vec![])));
    items.push(item("Copy Today's Date", "Copy the current date (YYYY-MM-DD) to clipboard",
        ICON_FILE, func("copy_date", vec![])));
    items.push(item("Copy Current Time", "Copy the current time (HH:MM:SS) to clipboard",
        ICON_FILE, func("copy_time", vec![])));
    items.push(item("Copy ISO Timestamp", "Copy the current date and time as an ISO 8601 timestamp",
        ICON_FILE, func("copy_datetime", vec![])));
    items.push(item("Copy Local IP", "Copy your local network IP address to clipboard",
        ICON_FILE, func("copy_local_ip", vec![])));
    items.push(item("Copy Hostname", "Copy this machine's hostname to clipboard",
        ICON_FILE, func("copy_hostname", vec![])));
    items.push(item("Copy Username", "Copy the current username to clipboard",
        ICON_FILE, func("copy_username", vec![])));

    // ── Audio ─────────────────────────────────────────────────────────────
    items.push(item("Mute / Unmute", "Toggle mute on the default audio sink - volume sound",
        ICON_SETTINGS, shell("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle")));
    items.push(item("Volume 50%", "Set system volume to 50%",
        ICON_SETTINGS, shell("wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.5")));
    items.push(item("Volume 100%", "Set system volume to 100%",
        ICON_SETTINGS, shell("wpctl set-volume @DEFAULT_AUDIO_SINK@ 1.0")));
    items.push(item("Toggle Pink Noise", "Play or stop pink noise for focus and masking background sounds",
        "icons/pink-noise.png", func("toggle_pink_noise", vec![])));
    items.push(item("Toggle Rain Noise", "Play or stop rain sounds for relaxation and focus",
        ICON_TRANSPARENT, func("toggle_rain_noise", vec![])));

    // ── Network ───────────────────────────────────────────────────────────
    items.push(item("Toggle WiFi", "Turn WiFi on or off via NetworkManager - wireless network",
        ICON_SETTINGS, shell("nmcli -t -f WIFI radio | grep -q enabled && nmcli radio wifi off || nmcli radio wifi on")));

    // ── Apps ──────────────────────────────────────────────────────────────
    items.push(item("Open Terminal", "Open a new terminal window",
        "icons/system/power.png", shell("xdg-terminal-exec || gnome-terminal || kitty || alacritty || xterm")));
    items.push(item("Open File Manager", "Open the default file manager",
        ICON_FOLDER, shell("xdg-open $HOME")));

    items
}

impl SearchProvider for ShortcutsSearcher {
    fn name(&self) -> String { "shortcuts".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        let results = if q.is_empty() { all_shortcuts() } else { self.fuzzy_filter(all_shortcuts(), q) };
        SearchResult::list(results)
    }
}
