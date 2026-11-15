use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("quarry/sidebar_widths.json")
}

pub fn load() -> HashMap<String, u32> {
    fs::read_to_string(path())
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

pub fn save(widths: &HashMap<String, u32>) -> Result<(), String> {
    let path = path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let contents = serde_json::to_vec_pretty(widths).map_err(|error| error.to_string())?;
    let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    if let Err(error) = fs::rename(&temporary, &path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(())
}
