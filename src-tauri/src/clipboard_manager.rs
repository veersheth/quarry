use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: u64,
}

impl ClipboardEntry {
    fn new(content: String) -> Self {
        Self {
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

pub struct ClipboardManager {
    history: Arc<Mutex<Vec<ClipboardEntry>>>,
    storage_path: Option<PathBuf>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(Vec::new())),
            storage_path: None,
        }
    }

    /// Create manager with persistent storage
    pub fn with_storage(storage_path: PathBuf) -> Self {
        let mut manager = Self {
            history: Arc::new(Mutex::new(Vec::new())),
            storage_path: Some(storage_path),
        };
        manager.load_from_disk();
        manager
    }

    fn load_from_disk(&mut self) {
        if let Some(path) = &self.storage_path {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(entries) = serde_json::from_str::<Vec<ClipboardEntry>>(&data) {
                    if let Ok(mut hist) = self.history.lock() {
                        *hist = entries;
                    }
                }
            }
        }
    }

    fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            if let Ok(hist) = self.history.lock() {
                if let Ok(json) = serde_json::to_string(&*hist) {
                    let _ = fs::write(path, json);
                }
            }
        }
    }

    pub fn start_monitoring(&self) {
        let history = Arc::clone(&self.history);
        let storage_path = self.storage_path.clone();

        thread::spawn(move || {
            let mut clipboard = arboard::Clipboard::new().expect("Failed to initialize clipboard");
            let mut last_content: Option<String> = None;

            loop {
                thread::sleep(Duration::from_millis(500));

                if let Ok(content) = clipboard.get_text() {
                    // Only add if content has changed
                    if last_content.as_ref() != Some(&content) {
                        if let Ok(mut hist) = history.lock() {
                            // Don't add empty strings or very large content (>1MB)
                            if !content.trim().is_empty() && content.len() < 1_000_000 {
                                // Check if this content already exists at the front
                                let should_add = hist.is_empty() || hist[0].content != content;

                                if should_add {
                                    let entry = ClipboardEntry::new(content.clone());
                                    hist.insert(0, entry);


                                    if storage_path.is_some() {
                                        drop(hist);
                                        if let Ok(hist) = history.lock() {
                                            if let Some(path) = &storage_path {
                                                if let Ok(json) = serde_json::to_string(&*hist) {
                                                    let _ = fs::write(path, json);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        last_content = Some(content);
                    }
                }
            }
        });
    }

    pub fn get_history(&self) -> Vec<ClipboardEntry> {
        self.history.lock().map(|h| h.clone()).unwrap_or_default()
    }

    pub fn clear_history(&self) {
        if let Ok(mut hist) = self.history.lock() {
            hist.clear();
        }
        self.save_to_disk();
    }

    pub fn remove_entry(&self, index: usize) {
        if let Ok(mut hist) = self.history.lock() {
            if index < hist.len() {
                hist.remove(index);
            }
        }
        self.save_to_disk();
    }
}
