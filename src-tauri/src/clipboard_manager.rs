use base64::{engine::general_purpose, Engine};
use image::{ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Max raw RGBA bytes (~4MB = ~1000x1000 image) before we skip storing
const MAX_IMAGE_BYTES: usize = 4_000_000;
// Max images kept in history
const MAX_IMAGE_ENTRIES: usize = 20;
// Thumbnail width in pixels for preview
const THUMBNAIL_WIDTH: u32 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClipboardContent {
    Text {
        value: String,
    },
    Image {
        /// Base64-encoded PNG thumbnail for UI display
        thumbnail: String,
        /// Base64-encoded full PNG for copying back to clipboard
        full: String,
        width: u32,
        height: u32,
        /// xxhash of raw bytes for deduplication
        hash: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub content: ClipboardContent,
    pub timestamp: u64,
}

impl ClipboardEntry {
    fn new_text(value: String) -> Self {
        Self {
            content: ClipboardContent::Text { value },
            timestamp: now_secs(),
        }
    }

    fn new_image(thumbnail: String, full: String, width: u32, height: u32, hash: u64) -> Self {
        Self {
            content: ClipboardContent::Image {
                thumbnail,
                full,
                width,
                height,
                hash,
            },
            timestamp: now_secs(),
        }
    }

    /// Returns a plain-text label for display / searching
    pub fn display_text(&self) -> String {
        match &self.content {
            ClipboardContent::Text { value } => value.clone(),
            ClipboardContent::Image { width, height, .. } => {
                format!("[Image {}×{}]", width, height)
            }
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Simple, fast non-cryptographic hash for image deduplication.
fn hash_bytes(data: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    h.finish()
}

/// Encode raw RGBA image data as a base64 PNG string.
fn encode_png_base64(
    bytes: &[u8],
    width: u32,
    height: u32,
    target_width: Option<u32>,
) -> Option<String> {
    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, bytes.to_vec())?;

    // Optionally resize
    let img = if let Some(tw) = target_width {
        if width > tw {
            let scale = tw as f32 / width as f32;
            let th = (height as f32 * scale) as u32;
            image::imageops::resize(&img, tw, th, image::imageops::FilterType::Triangle)
        } else {
            img
        }
    } else {
        img
    };

    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .ok()?;

    Some(format!(
        "data:image/png;base64,{}",
        general_purpose::STANDARD.encode(buf.get_ref())
    ))
}

// ---------------------------------------------------------------------------

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

    fn save_to_disk_inner(history: &Arc<Mutex<Vec<ClipboardEntry>>>, path: &PathBuf) {
        if let Ok(hist) = history.lock() {
            if let Ok(json) = serde_json::to_string(&*hist) {
                let _ = fs::write(path, json);
            }
        }
    }

    fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            Self::save_to_disk_inner(&self.history, path);
        }
    }

    pub fn start_monitoring(&self) {
        let history = Arc::clone(&self.history);
        let storage_path = self.storage_path.clone();

        thread::spawn(move || {
            let mut clipboard =
                arboard::Clipboard::new().expect("Failed to initialize clipboard");

            let mut last_text: Option<String> = None;
            let mut last_image_hash: Option<u64> = None;

            loop {
                thread::sleep(Duration::from_millis(500));

                // --- Text ---
                if let Ok(text) = clipboard.get_text() {
                    if last_text.as_ref() != Some(&text) {
                        if !text.trim().is_empty() && text.len() < 1_000_000 {
                            let mut hist = history.lock().unwrap();
                            let is_new = hist
                                .first()
                                .map(|e| match &e.content {
                                    ClipboardContent::Text { value } => value != &text,
                                    _ => true,
                                })
                                .unwrap_or(true);

                            if is_new {
                                hist.insert(0, ClipboardEntry::new_text(text.clone()));
                                drop(hist);
                                if let Some(ref p) = storage_path {
                                    Self::save_to_disk_inner(&history, p);
                                }
                            }
                        }
                        last_text = Some(text);
                        // Clear image tracking so an image copied after this text is detected
                        last_image_hash = None;
                        continue; // clipboard holds text, skip image check this tick
                    }
                }

                // --- Image ---
                if let Ok(img_data) = clipboard.get_image() {
                    let raw: &[u8] = &img_data.bytes;
                    let w = img_data.width as u32;
                    let h = img_data.height as u32;

                    // Skip enormous images
                    if raw.len() > MAX_IMAGE_BYTES {
                        continue;
                    }

                    let hash = hash_bytes(raw);

                    if last_image_hash == Some(hash) {
                        continue; // same image, nothing to do
                    }
                    last_image_hash = Some(hash);
                    last_text = None; // reset text tracking

                    // Build thumbnail and full PNG on this thread (CPU-bound but rare)
                    let thumbnail =
                        match encode_png_base64(raw, w, h, Some(THUMBNAIL_WIDTH)) {
                            Some(t) => t,
                            None => continue,
                        };
                    let full = match encode_png_base64(raw, w, h, None) {
                        Some(f) => f,
                        None => continue,
                    };

                    let mut hist = history.lock().unwrap();

                    // Deduplicate: if the most recent entry is the same image, skip
                    let already_present = hist.first().map(|e| match &e.content {
                        ClipboardContent::Image { hash: h, .. } => *h == hash,
                        _ => false,
                    }).unwrap_or(false);

                    if already_present {
                        continue;
                    }

                    hist.insert(0, ClipboardEntry::new_image(thumbnail, full, w, h, hash));

                    // Enforce image cap: remove oldest image entries beyond the limit
                    let mut image_count = 0usize;
                    hist.retain(|e| {
                        if matches!(e.content, ClipboardContent::Image { .. }) {
                            image_count += 1;
                            image_count <= MAX_IMAGE_ENTRIES
                        } else {
                            true
                        }
                    });

                    drop(hist);
                    if let Some(ref p) = storage_path {
                        Self::save_to_disk_inner(&history, p);
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
