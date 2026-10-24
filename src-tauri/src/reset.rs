use once_cell::sync::Lazy;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const CONFIRMATION_WORDS: [&str; 128] = [
    "acorn", "amber", "anchor", "apricot", "ash", "atlas", "autumn", "badger", "bamboo", "beacon",
    "berry", "birch", "bison", "bluejay", "boulder", "breeze", "brook", "cactus", "candle",
    "canyon", "caraway", "cedar", "cherry", "cinder", "clover", "comet", "copper", "coral",
    "cricket", "cypress", "dahlia", "dawn", "delta", "dew", "dolphin", "drift", "eagle", "earth",
    "echo", "ember", "falcon", "fern", "finch", "fjord", "flint", "forest", "fossil", "frost",
    "garden", "ginger", "glacier", "goblet", "granite", "harbor", "hazel", "heron", "hickory",
    "honey", "horizon", "island", "ivory", "jasmine", "jigsaw", "juniper", "kestrel", "lagoon",
    "lantern", "laurel", "lemon", "lichen", "lotus", "maple", "marble", "meadow", "meteor",
    "midnight", "minnow", "moss", "mountain", "nectar", "oasis", "oat", "ocean", "olive", "onyx",
    "orbit", "orchid", "otter", "paddle", "pebble", "pepper", "petal", "phoenix", "pine", "planet",
    "plum", "pocket", "prairie", "quartz", "rabbit", "raven", "ripple", "river", "robin",
    "saffron", "sailor", "sequoia", "shadow", "silver", "spruce", "starling", "stone", "summit",
    "thistle", "thunder", "timber", "topaz", "valley", "violet", "willow", "winter", "wren",
    "yellow", "zenith", "zinnia", "acoustic", "bramble", "cascade",
];

static RESET_CHALLENGE: Lazy<Mutex<Option<(String, Instant)>>> = Lazy::new(|| Mutex::new(None));
const CHALLENGE_LIFETIME: Duration = Duration::from_secs(10 * 60);

#[tauri::command]
pub fn create_reset_confirmation() -> Result<String, String> {
    let mut entropy = [0u8; 8];
    File::open("/dev/urandom")
        .and_then(|mut random| random.read_exact(&mut entropy))
        .map_err(|error| format!("Could not generate a secure confirmation phrase: {error}"))?;

    let phrase = entropy
        .iter()
        .map(|byte| CONFIRMATION_WORDS[usize::from(*byte & 0x7f)])
        .collect::<Vec<_>>()
        .join(" ");
    let mut challenge = RESET_CHALLENGE
        .lock()
        .map_err(|_| "Reset confirmation is unavailable".to_string())?;
    *challenge = Some((phrase.clone(), Instant::now()));
    Ok(phrase)
}

#[tauri::command]
pub fn cancel_reset_confirmation() {
    if let Ok(mut challenge) = RESET_CHALLENGE.lock() {
        *challenge = None;
    }
}

#[tauri::command]
pub fn reset_quarry(phrase: String, app: AppHandle) -> Result<(), String> {
    let expected = {
        let mut challenge = RESET_CHALLENGE
            .lock()
            .map_err(|_| "Reset confirmation is unavailable".to_string())?;
        let Some((expected, created)) = challenge.take() else {
            return Err("Generate a new confirmation phrase and try again".into());
        };
        if created.elapsed() > CHALLENGE_LIFETIME {
            return Err("The confirmation phrase expired. Generate a new one".into());
        }
        expected
    };

    if normalize_phrase(&phrase) != normalize_phrase(&expected) {
        return Err("The confirmation phrase did not match. Generate a new one to retry".into());
    }

    crate::searchers::dictionary::begin_reset()?;
    let result = reset_owned_data(app);
    crate::searchers::dictionary::end_reset();
    result
}

fn reset_owned_data(app: AppHandle) -> Result<(), String> {
    let config_dir = crate::config::Config::config_path()
        .parent()
        .ok_or_else(|| "Could not determine Quarry's config directory".to_string())?
        .to_path_buf();
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not determine Quarry's data directory".to_string())?
        .join("quarry");
    ensure_safe_app_directory(&config_dir)?;
    ensure_safe_app_directory(&data_dir)?;

    let mut files_to_remove = vec![
        config_dir.join("bookmarks.json"),
        config_dir.join("pins.json"),
        config_dir.join("scratch.txt"),
        data_dir.join("dictionary.sqlite"),
        data_dir.join("dictionary.sqlite.new"),
        data_dir.join("dictionary.etag"),
        data_dir.join("dictionary.checked"),
        data_dir.join(".groq_key"),
        data_dir.join("usage.json"),
        data_dir.join("timers.json"),
        data_dir.join("calc_history.json"),
    ];
    // Config::save writes fresh defaults after the existing file is removed.
    files_to_remove.push(crate::config::Config::config_path());
    for file in &files_to_remove {
        validate_owned_file(file)?;
    }
    crate::CLIPBOARD_MANAGER.validate_reset_storage()?;
    crate::CLIPBOARD_MANAGER.stop_monitoring();
    crate::searchers::files::stop_file_index();
    for file in &files_to_remove {
        remove_owned_file(file)?;
    }

    crate::CLIPBOARD_MANAGER.clear_history_for_reset()?;
    crate::searchers::math::clear_history();
    crate::searchers::timer::clear_timers();
    crate::PINS.clear();
    if let Ok(mut usage) = crate::USAGE_HISTORY.write() {
        usage.clear();
    }

    let mut defaults = crate::config::Config::default();
    defaults.setup_completed = false;
    crate::config::Config::save(&defaults)?;
    if let Ok(mut config) = crate::CONFIG.write() {
        *config = defaults;
    }
    crate::commands::reload_triggers();

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.eval("location.reload()");
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(())
}

fn normalize_phrase(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn ensure_safe_app_directory(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "Refusing to reset through a symlink at {}",
            path.display()
        )),
        Ok(metadata) if !metadata.is_dir() => {
            Err(format!("Expected a Quarry directory at {}", path.display()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Could not inspect {}: {error}", path.display())),
    }
}

fn validate_owned_file(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Err(format!(
            "Refusing to remove a directory during reset: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Could not inspect {}: {error}", path.display())),
    }
}

fn remove_owned_file(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => fs::remove_file(path)
            .map_err(|error| format!("Could not remove {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Could not inspect {}: {error}", path.display())),
    }
}
