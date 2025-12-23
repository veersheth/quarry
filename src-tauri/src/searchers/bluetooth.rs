use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;
use std::process::Command;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

pub struct BluetoothSearcher;

#[derive(Debug, Clone)]
struct BluetoothDevice {
    mac: String,
    name: String,
    paired: bool,
    connected: bool,
}

struct ScanCache {
    devices: Vec<BluetoothDevice>,
    last_scan: Option<Instant>,
}

static SCAN_CACHE: Lazy<Mutex<ScanCache>> = Lazy::new(|| {
    Mutex::new(ScanCache {
        devices: Vec::new(),
        last_scan: None,
    })
});

impl BluetoothSearcher {
    fn is_bluetooth_on() -> bool {
        Command::new("bluetoothctl")
            .arg("show")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines()
                    .find(|line| line.trim().starts_with("Powered:"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|val| val.trim() == "yes")
            })
            .unwrap_or(false)
    }

    fn get_paired_devices() -> Vec<BluetoothDevice> {
        let output = Command::new("bluetoothctl")
            .arg("devices")
            .arg("Paired")
            .output();

        let Ok(output) = output else {
            return Vec::new();
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines() {
            // Format: "Device XX:XX:XX:XX:XX:XX Device Name"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0] == "Device" {
                let mac = parts[1].to_string();
                let name = parts[2..].join(" ");
                
                // Check if connected
                let connected = Self::is_device_connected(&mac);
                
                devices.push(BluetoothDevice {
                    mac,
                    name,
                    paired: true,
                    connected,
                });
            }
        }

        devices
    }

    fn is_device_connected(mac: &str) -> bool {
        Command::new("bluetoothctl")
            .arg("info")
            .arg(mac)
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines()
                    .find(|line| line.trim().starts_with("Connected:"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|val| val.trim() == "yes")
            })
            .unwrap_or(false)
    }

    fn scan_nearby_devices() -> Vec<BluetoothDevice> {
        // Check cache first
        if let Ok(cache) = SCAN_CACHE.lock() {
            if let Some(last_scan) = cache.last_scan {
                // Use cached results if less than 5 seconds old
                if last_scan.elapsed() < Duration::from_secs(5) {
                    return cache.devices.clone();
                }
            }
        }

        // Start fresh scan in background
        std::thread::spawn(|| {
            use std::io::{BufRead, BufReader};
            use std::process::Stdio;

            // Start bluetoothctl in interactive mode
            let mut child = match Command::new("bluetoothctl")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => return,
            };

            let stdin = child.stdin.as_mut().unwrap();
            let stdout = child.stdout.take().unwrap();
            
            // Send scan on command
            use std::io::Write;
            let _ = stdin.write_all(b"scan on\n");
            let _ = stdin.flush();

            // Read streaming output for 3 seconds
            let reader = BufReader::new(stdout);
            let mut devices = Vec::new();
            let paired_macs: Vec<String> = BluetoothSearcher::get_paired_devices()
                .iter()
                .map(|d| d.mac.clone())
                .collect();

            let start = Instant::now();
            for line in reader.lines() {
                if start.elapsed() > Duration::from_secs(3) {
                    break;
                }

                let Ok(line) = line else { continue };
                
                // Look for [NEW] Device lines
                // Format: "[NEW] Device 84:5F:04:C8:88:86 Galaxy Buds2 (8886)"
                if line.contains("[NEW] Device") || line.contains("[CHG] Device") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let mac = parts[2].to_string();
                        
                        // Skip if already paired
                        if paired_macs.contains(&mac) {
                            continue;
                        }
                        
                        // Get device name (everything after MAC address)
                        let name = if parts.len() > 3 {
                            parts[3..].join(" ")
                        } else {
                            mac.clone()
                        };

                        // Avoid duplicates
                        if !devices.iter().any(|d: &BluetoothDevice| d.mac == mac) {
                            devices.push(BluetoothDevice {
                                mac,
                                name,
                                paired: false,
                                connected: false,
                            });
                        }
                    }
                }
            }

            // Update cache
            if let Ok(mut cache) = SCAN_CACHE.lock() {
                cache.devices = devices;
                cache.last_scan = Some(Instant::now());
            }

            // Clean up: send scan off and exit
            let _ = stdin.write_all(b"scan off\nexit\n");
            let _ = child.wait();
        });

        // Return cached results immediately (might be empty on first call)
        SCAN_CACHE
            .lock()
            .map(|cache| cache.devices.clone())
            .unwrap_or_default()
    }
}

impl SearchProvider for BluetoothSearcher {
    fn search(&self, _query: &str, _app: &AppHandle) -> SearchResult {
        let mut results = Vec::new();
        let bt_on = Self::is_bluetooth_on();

        // Power toggle option
        if bt_on {
            let action_id = "bt_power_off".to_string();
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::ShellCommand {
                        command: "bluetoothctl power off".to_string(),
                    },
                );
            }
            results.push(ResultItem {
                name: "Turn Bluetooth Off".to_string(),
                action_id,
                description: Some("Disable Bluetooth adapter".to_string()),
                icon: None,
            });
        } else {
            let action_id = "bt_power_on".to_string();
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::ShellCommand {
                        command: "bluetoothctl power on".to_string(),
                    },
                );
            }
            results.push(ResultItem {
                name: "Turn Bluetooth On".to_string(),
                action_id,
                description: Some("Enable Bluetooth adapter".to_string()),
                icon: None,
            });
        }

        // Only show devices if Bluetooth is on
        if bt_on {
            // Paired devices
            let paired = Self::get_paired_devices();
            for device in paired {
                let status = if device.connected {
                    "Connected"
                } else {
                    "Paired"
                };

                if device.connected {
                    // Disconnect option
                    let action_id = format!("bt_disconnect_{}", device.mac);
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::ShellCommand {
                                command: format!("bluetoothctl disconnect {}", device.mac),
                            },
                        );
                    }
                    results.push(ResultItem {
                        name: format!("{} - Disconnect", device.name),
                        action_id,
                        description: Some(format!("{} • {}", status, device.mac)),
                        icon: None,
                    });
                } else {
                    // Connect option
                    let action_id = format!("bt_connect_{}", device.mac);
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::ShellCommand {
                                command: format!("bluetoothctl connect {}", device.mac),
                            },
                        );
                    }
                    results.push(ResultItem {
                        name: format!("{} - Connect", device.name),
                        action_id,
                        description: Some(format!("{} • {}", status, device.mac)),
                        icon: None,
                    });
                }
            }

            // Nearby devices - use trust + pair + connect approach
            let nearby = Self::scan_nearby_devices();
            if !nearby.is_empty() {
                for device in nearby {
                    let action_id = format!("bt_pair_{}", device.mac);
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        // Trust the device first to auto-accept pairing
                        // Then pair and connect
                        registry.register(
                            action_id.clone(),
                            ActionData::ShellCommand {
                                command: format!(
                                    "bluetoothctl trust {} && bluetoothctl pair {} && bluetoothctl connect {}",
                                    device.mac, device.mac, device.mac
                                ),
                            },
                        );
                    }
                    results.push(ResultItem {
                        name: format!("{} - Pair & Connect", device.name),
                        action_id,
                        description: Some(format!("Nearby • {}", device.mac)),
                        icon: None,
                    });
                }
            }
        }

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
