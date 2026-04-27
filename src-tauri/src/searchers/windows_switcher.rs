use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct WindowEntry {
    pub title:     String,
    pub class:     String,
    pub focus_cmd: String,
    pub workspace: String,
}

struct Cache {
    windows:    Vec<WindowEntry>,
    fetched_at: Instant,
}

static CACHE: Lazy<Mutex<Option<Cache>>> = Lazy::new(|| Mutex::new(None));
const TTL: Duration = Duration::from_millis(500);

pub fn get_windows() -> Vec<WindowEntry> {
    if let Ok(mut guard) = CACHE.lock() {
        if let Some(ref c) = *guard {
            if c.fetched_at.elapsed() < TTL {
                return c.windows.iter().map(|w| WindowEntry {
                    title:     w.title.clone(),
                    class:     w.class.clone(),
                    focus_cmd: w.focus_cmd.clone(),
                    workspace: w.workspace.clone(),
                }).collect();
            }
        }
        let windows = fetch_windows();
        *guard = Some(Cache { windows: windows.iter().map(|w| WindowEntry {
            title:     w.title.clone(),
            class:     w.class.clone(),
            focus_cmd: w.focus_cmd.clone(),
            workspace: w.workspace.clone(),
        }).collect(), fetched_at: Instant::now() });
        windows
    } else {
        fetch_windows()
    }
}

fn fetch_windows() -> Vec<WindowEntry> {
    // Try Hyprland
    if let Ok(out) = Command::new("hyprctl").args(["clients", "-j"]).output() {
        if out.status.success() {
            if let Ok(text) = std::str::from_utf8(&out.stdout) {
                if let Some(windows) = parse_hyprland(text) {
                    return windows;
                }
            }
        }
    }

    // Try Sway
    if let Ok(out) = Command::new("swaymsg").args(["-t", "get_tree"]).output() {
        if out.status.success() {
            if let Ok(text) = std::str::from_utf8(&out.stdout) {
                if let Some(windows) = parse_sway(text) {
                    return windows;
                }
            }
        }
    }

    // Try wmctrl (X11)
    if let Ok(out) = Command::new("wmctrl").args(["-lx"]).output() {
        if out.status.success() {
            if let Ok(text) = std::str::from_utf8(&out.stdout) {
                return parse_wmctrl(text);
            }
        }
    }

    vec![]
}

fn parse_hyprland(text: &str) -> Option<Vec<WindowEntry>> {
    use serde_json::Value;
    let arr: Vec<Value> = serde_json::from_str(text).ok()?;
    let windows = arr.iter().filter_map(|v| {
        let title   = v["title"].as_str()?.to_string();
        let class   = v["class"].as_str().unwrap_or("").to_string();
        let addr    = v["address"].as_str().unwrap_or("").to_string();
        let ws_name = v["workspace"]["name"].as_str().unwrap_or("").to_string();
        if title.is_empty() { return None; }
        Some(WindowEntry {
            title,
            class,
            workspace: ws_name,
            focus_cmd: format!("hyprctl dispatch focuswindow address:{}", addr),
        })
    }).collect();
    Some(windows)
}

fn parse_sway(text: &str) -> Option<Vec<WindowEntry>> {
    use serde_json::Value;
    let root: Value = serde_json::from_str(text).ok()?;
    let mut windows = vec![];
    collect_sway_windows(&root, &mut windows);
    Some(windows)
}

fn collect_sway_windows(node: &serde_json::Value, out: &mut Vec<WindowEntry>) {
    if node["type"] == "con" {
        if let Some(name) = node["name"].as_str() {
            if !name.is_empty() && node["pid"].as_u64().is_some() {
                let id = node["id"].as_u64().unwrap_or(0);
                let class = node["window_properties"]["class"]
                    .as_str().unwrap_or("").to_string();
                let ws = "".to_string();
                out.push(WindowEntry {
                    title:     name.to_string(),
                    class,
                    workspace: ws,
                    focus_cmd: format!("swaymsg '[con_id={}] focus'", id),
                });
            }
        }
    }
    if let Some(nodes) = node["nodes"].as_array() {
        for n in nodes { collect_sway_windows(n, out); }
    }
    if let Some(nodes) = node["floating_nodes"].as_array() {
        for n in nodes { collect_sway_windows(n, out); }
    }
}

fn parse_wmctrl(text: &str) -> Vec<WindowEntry> {
    text.lines().filter_map(|line| {
        let parts: Vec<&str> = line.splitn(5, ' ').collect();
        if parts.len() < 5 { return None; }
        let hex_id = parts[0];
        let class  = parts[2].to_string();
        let title  = parts[4].trim().to_string();
        if title.is_empty() { return None; }
        Some(WindowEntry {
            title,
            class,
            workspace: String::new(),
            focus_cmd: format!("wmctrl -ia {}", hex_id),
        })
    }).collect()
}

// ─────────────────────────────────────────────────────────────
// Searcher
// ─────────────────────────────────────────────────────────────

pub struct WindowSwitcher;

impl SearchProvider for WindowSwitcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let windows = get_windows();

        let q = query.trim().to_lowercase();
        let mut results: Vec<ResultItem> = windows.into_iter()
            .filter(|w| {
                q.is_empty()
                    || w.title.to_lowercase().contains(&q)
                    || w.class.to_lowercase().contains(&q)
            })
            .map(|w| {
                let desc = if w.workspace.is_empty() {
                    w.class.clone()
                } else {
                    format!("{} · {}", w.class, w.workspace)
                };
                ResultItem::new(
                    w.title.clone(),
                    vec![Action::new("Focus", ActionData::ShellCommand {
                        command: w.focus_cmd.clone(),
                    })],
                ).description(desc)
            })
            .collect();

        if results.is_empty() {
            results.push(
                ResultItem::new("No open windows found", vec![])
                    .description("Make sure hyprctl, swaymsg, or wmctrl is available"),
            );
        }

        SearchResult { results, result_type: ResultType::List }
    }
}
