use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;
use std::process::Command;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

pub struct WifiSearcher;

#[derive(Debug, Clone)]
struct WifiNetwork {
    ssid: String,
    signal_strength: i32,
    security: String,
    connected: bool,
    saved: bool,
}

struct ScanCache {
    networks: Vec<WifiNetwork>,
    last_scan: Option<Instant>,
}

static SCAN_CACHE: Lazy<Mutex<ScanCache>> = Lazy::new(|| {
    Mutex::new(ScanCache {
        networks: Vec::new(),
        last_scan: None,
    })
});

impl WifiSearcher {
    fn is_wifi_on() -> bool {
        Command::new("nmcli")
            .arg("radio")
            .arg("wifi")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Some(stdout.trim() == "enabled")
            })
            .unwrap_or(false)
    }

    fn get_current_connection() -> Option<String> {
        let output = Command::new("nmcli")
            .arg("-t")
            .arg("-f")
            .arg("NAME,TYPE,DEVICE")
            .arg("connection")
            .arg("show")
            .arg("--active")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[1] == "802-11-wireless" {
                return Some(parts[0].to_string());
            }
        }
        None
    }

    fn get_saved_networks() -> Vec<String> {
        let output = Command::new("nmcli")
            .arg("-t")
            .arg("-f")
            .arg("NAME,TYPE")
            .arg("connection")
            .arg("show")
            .output();

        let Ok(output) = output else {
            return Vec::new();
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 && parts[1] == "802-11-wireless" {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn scan_networks() -> Vec<WifiNetwork> {
        // Check cache first
        if let Ok(cache) = SCAN_CACHE.lock() {
            if let Some(last_scan) = cache.last_scan {
                // Use cached results if less than 5 seconds old
                if last_scan.elapsed() < Duration::from_secs(5) {
                    return cache.networks.clone();
                }
            }
        }

        // Start fresh scan in background
        std::thread::spawn(|| {
            // Trigger a rescan
            let _ = Command::new("nmcli")
                .arg("device")
                .arg("wifi")
                .arg("rescan")
                .output();

            // Wait a moment for scan to complete
            std::thread::sleep(Duration::from_millis(1500));

            // Get scan results
            let output = Command::new("nmcli")
                .arg("-t")
                .arg("-f")
                .arg("SSID,SIGNAL,SECURITY,IN-USE")
                .arg("device")
                .arg("wifi")
                .arg("list")
                .output();

            let Ok(output) = output else {
                return;
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            let saved_networks = Self::get_saved_networks();
            let mut networks = Vec::new();
            let mut seen_ssids = std::collections::HashSet::new();

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let ssid = parts[0].to_string();
                    
                    // Skip empty SSIDs and duplicates
                    if ssid.is_empty() || seen_ssids.contains(&ssid) {
                        continue;
                    }
                    seen_ssids.insert(ssid.clone());

                    let signal_strength = parts[1].parse::<i32>().unwrap_or(0);
                    let security = parts[2].to_string();
                    let connected = parts[3] == "*";
                    let saved = saved_networks.contains(&ssid);

                    networks.push(WifiNetwork {
                        ssid,
                        signal_strength,
                        security,
                        connected,
                        saved,
                    });
                }
            }

            // Sort by signal strength
            networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));

            // Update cache
            if let Ok(mut cache) = SCAN_CACHE.lock() {
                cache.networks = networks;
                cache.last_scan = Some(Instant::now());
            }
        });

        // Return cached results immediately (might be empty on first call)
        SCAN_CACHE
            .lock()
            .map(|cache| cache.networks.clone())
            .unwrap_or_default()
    }

    fn get_signal_icon(strength: i32) -> &'static str {
        match strength {
            80..=100 => "📶",
            60..=79 => "📶",
            40..=59 => "📶",
            20..=39 => "📶",
            _ => "📶",
        }
    }

    fn get_security_icon(security: &str) -> &'static str {
        if security.is_empty() || security == "--" {
            "🔓" // Open network
        } else {
            "🔒" // Secured network
        }
    }
}

impl SearchProvider for WifiSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query_lower = query.trim().to_lowercase();
        let mut results = Vec::new();
        let wifi_on = Self::is_wifi_on();

        // Power toggle option
        if wifi_on {
            let action_id = "wifi_power_off".to_string();
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::ShellCommand {
                        command: "nmcli radio wifi off".to_string(),
                    },
                );
            }
            results.push(ResultItem {
                name: "Turn Wifi Off".to_string(),
                action_id,
                description: Some("Disable Wifi adapter".to_string()),
                icon: None,
            });
        } else {
            let action_id = "wifi_power_on".to_string();
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::ShellCommand {
                        command: "nmcli radio wifi on".to_string(),
                    },
                );
            }
            results.push(ResultItem {
                name: "Turn Wifi On".to_string(),
                action_id,
                description: Some("Enable Wifi adapter".to_string()),
                icon: None,
            });
        }

        // Only show networks if Wifi is on
        if wifi_on {
            let current_connection = Self::get_current_connection();
            let networks = Self::scan_networks();

            for network in networks {
                // Apply query filter if provided
                if !query_lower.is_empty() && !network.ssid.to_lowercase().contains(&query_lower) {
                    continue;
                }

                let signal_icon = Self::get_signal_icon(network.signal_strength);
                let security_icon = Self::get_security_icon(&network.security);

                if network.connected {
                    // Disconnect option for current network
                    let action_id = format!("wifi_disconnect_{}", network.ssid);
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::ShellCommand {
                                command: format!("nmcli connection down \"{}\"", network.ssid),
                            },
                        );
                    }
                    results.push(ResultItem {
                        name: format!("{} - Disconnect", network.ssid),
                        action_id,
                        description: Some(format!(
                            "🔵 Connected • {} {} {}%",
                            signal_icon, security_icon, network.signal_strength
                        )),
                        icon: None,
                    });
                } else if network.saved {
                    // Connect to saved network
                    let action_id = format!("wifi_connect_{}", network.ssid);
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::ShellCommand {
                                command: format!("nmcli connection up \"{}\"", network.ssid),
                            },
                        );
                    }
                    results.push(ResultItem {
                        name: format!("{} - Connect", network.ssid),
                        action_id,
                        description: Some(format!(
                            "Saved • {} {} {}%",
                            signal_icon, security_icon, network.signal_strength
                        )),
                        icon: None,
                    });
                } else {
                    // Connect to new network
                    let action_id = format!("wifi_new_{}", network.ssid);
                    
                    let command = if network.security.is_empty() || network.security == "--" {
                        // Open network - connect directly
                        format!("nmcli device wifi connect \"{}\"", network.ssid)
                    } else {
                        // Secured network - will prompt for password
                        // Using --ask to prompt for password
                        format!("nmcli --ask device wifi connect \"{}\"", network.ssid)
                    };

                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::ShellCommand { command },
                        );
                    }
                    results.push(ResultItem {
                        name: format!("{} - Connect", network.ssid),
                        action_id,
                        description: Some(format!(
                            "New • {} {} {}%",
                            signal_icon, security_icon, network.signal_strength
                        )),
                        icon: None,
                    });
                }
            }
        }

        SearchResult {
            results,
            result_type: ResultType::List,
            usage_sorted: false,
            additional_info: Some("Note: Requires NetworkManager (nmcli). Press <space> to refresh scan".to_string()),
        }
    }
}
