use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinEntry {
    pub name: String,    // stable dedup key
    pub payload: String, // searcher-specific data to reconstruct the item
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PinData {
    pins: HashMap<String, Vec<PinEntry>>,
}

pub struct PinStore {
    data: RwLock<PinData>,
}

impl PinStore {
    fn path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("quarry")
            .join("pins.json")
    }

    pub fn load() -> Self {
        let data = std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { data: RwLock::new(data) }
    }

    pub fn get(&self, searcher: &str) -> Vec<PinEntry> {
        self.data.read().unwrap_or_else(|e| e.into_inner())
            .pins.get(searcher).cloned().unwrap_or_default()
    }

    pub fn add(&self, searcher: &str, entry: PinEntry) {
        {
            let mut data = self.data.write().unwrap_or_else(|e| e.into_inner());
            let entries = data.pins.entry(searcher.to_string()).or_default();
            if !entries.iter().any(|e| e.name == entry.name) {
                entries.push(entry);
            }
        }
        self.save();
    }

    pub fn remove(&self, searcher: &str, name: &str) {
        {
            let mut data = self.data.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entries) = data.pins.get_mut(searcher) {
                entries.retain(|e| e.name != name);
            }
        }
        self.save();
    }

    pub fn contains(&self, searcher: &str, name: &str) -> bool {
        self.data.read().unwrap_or_else(|e| e.into_inner())
            .pins.get(searcher)
            .map(|v| v.iter().any(|e| e.name == name))
            .unwrap_or(false)
    }

    fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(data) = self.data.read() {
            if let Ok(json) = serde_json::to_string_pretty(&*data) {
                std::fs::write(&path, json).ok();
            }
        }
    }
}
