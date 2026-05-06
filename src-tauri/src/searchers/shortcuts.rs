use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct ShortcutsSearcher;

fn item(name: &str, description: &str, icon: &str, action: ActionData) -> ResultItem {
    ResultItem::new(name, vec![Action::new("Run", action)])
        .description(description)
        .icon(icon)
}

fn folder_item(name: &str, description: &str, path: std::path::PathBuf) -> ResultItem {
    let url = format!("file://{}", path.to_string_lossy());
    item(name, description, "icons/folder.png", ActionData::OpenUrl { url })
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
    items.push(item(
        "Reload Quarry",
        "Rebuild file index, refresh app list, reload styles",
        "icon-transparent.png",
        func("reload_quarry", vec![]),
    ));
    items.push(item(
        "Open Quarry Settings",
        "Open the quarry settings window",
        "icon-transparent.png",
        func("open_settings", vec![]),
    ));
    if let Some(cfg) = dirs::config_dir() {
        let mut it = folder_item(
            "Open Quarry Config Folder",
            "Open ~/.config/quarry in your file manager",
            cfg.join("quarry"),
        );
        it.icon = Some("icon-transparent.png".into());
        items.push(it);
    }

    // ── Trash ─────────────────────────────────────────────────────────────
    items.push(item(
        "Open Trash",
        "Open the trash folder - deleted files recycle bin",
        "icons/folder.png",
        shell("gio open trash://"),
    ));
    items.push(item(
        "Empty Trash",
        "Permanently delete all trashed files - clear recycle bin",
        "icons/folder.png",
        ActionData::RunFunction {
            function_name: "show_modal".into(),
            params: vec![
                "## Empty Trash\n\nThis will permanently delete all trashed files.".into(),
                "Empty Trash|danger|gio trash --empty".into(),
                "Cancel|default|".into(),
            ],
        },
    ));

    // ── Files ─────────────────────────────────────────────────────────────
    items.push(item(
        "Open Latest Download",
        "Open the most recently downloaded file",
        "icons/file.png",
        func("open_latest_download", vec![]),
    ));
    items.push(item(
        "Show Latest Download",
        "Open the Downloads folder with the latest file selected",
        "icons/folder.png",
        func("show_latest_download", vec![]),
    ));
    items.push(item(
        "Open Latest Screenshot",
        "Open the most recently taken screenshot",
        "icons/file.png",
        func("open_latest_screenshot", vec![]),
    ));

    // ── Folders ───────────────────────────────────────────────────────────
    if let Some(p) = dirs::download_dir() {
        items.push(folder_item("Open Downloads", "Open the Downloads folder", p));
    }
    if let Some(p) = dirs::home_dir() {
        items.push(folder_item("Open Home Folder", "Open your home directory", p));
    }
    if let Some(p) = dirs::picture_dir() {
        items.push(folder_item("Open Pictures", "Open the Pictures folder", p));
    }
    if let Some(p) = dirs::document_dir() {
        items.push(folder_item("Open Documents", "Open the Documents folder", p));
    }
    if let Some(p) = dirs::desktop_dir() {
        items.push(folder_item("Open Desktop", "Open the Desktop folder", p));
    }

    // ── Copy to clipboard ─────────────────────────────────────────────────
    items.push(item(
        "Copy Today's Date",
        "Copy the current date (YYYY-MM-DD) to clipboard",
        "icons/file.png",
        func("copy_date", vec![]),
    ));
    items.push(item(
        "Copy Current Time",
        "Copy the current time (HH:MM:SS) to clipboard",
        "icons/file.png",
        func("copy_time", vec![]),
    ));
    items.push(item(
        "Copy ISO Timestamp",
        "Copy the current date and time as an ISO 8601 timestamp",
        "icons/file.png",
        func("copy_datetime", vec![]),
    ));
    items.push(item(
        "Copy Local IP",
        "Copy your local network IP address to clipboard",
        "icons/file.png",
        func("copy_local_ip", vec![]),
    ));
    items.push(item(
        "Copy Hostname",
        "Copy this machine's hostname to clipboard",
        "icons/file.png",
        func("copy_hostname", vec![]),
    ));
    items.push(item(
        "Copy Username",
        "Copy the current username to clipboard",
        "icons/file.png",
        func("copy_username", vec![]),
    ));

    // ── Audio ─────────────────────────────────────────────────────────────
    items.push(item(
        "Mute / Unmute",
        "Toggle mute on the default audio sink - volume sound",
        "icons/system/power.png",
        shell("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"),
    ));
    items.push(item(
        "Volume 50%",
        "Set system volume to 50%",
        "icons/system/power.png",
        shell("wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.5"),
    ));
    items.push(item(
        "Volume 100%",
        "Set system volume to 100%",
        "icons/system/power.png",
        shell("wpctl set-volume @DEFAULT_AUDIO_SINK@ 1.0"),
    ));

    items.push(item(
        "Toggle Pink Noise",
        "Play or stop pink noise for focus and masking background sounds",
        "icons/system/power.png",
        func("toggle_pink_noise", vec![]),
    ));

    // ── Network ───────────────────────────────────────────────────────────
    items.push(item(
        "Toggle WiFi",
        "Turn WiFi on or off via NetworkManager - wireless network",
        "icons/system/power.png",
        shell("nmcli -t -f WIFI radio | grep -q enabled && nmcli radio wifi off || nmcli radio wifi on"),
    ));

    // ── Apps ──────────────────────────────────────────────────────────────
    items.push(item(
        "Open Terminal",
        "Open a new terminal window",
        "icons/system/power.png",
        shell("xdg-terminal-exec || gnome-terminal || kitty || alacritty || xterm"),
    ));
    items.push(item(
        "Open File Manager",
        "Open the default file manager",
        "icons/folder.png",
        shell("xdg-open $HOME"),
    ));

    items
}

impl SearchProvider for ShortcutsSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        let results = if q.is_empty() {
            all_shortcuts()
        } else {
            self.fuzzy_filter(all_shortcuts(), q)
        };
        SearchResult { results, result_type: ResultType::List }
    }
}
