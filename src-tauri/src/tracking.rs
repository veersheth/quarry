use once_cell::sync::Lazy;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static CLIPBOARD_ENABLED: Lazy<AtomicBool> =
    Lazy::new(|| AtomicBool::new(read_flag("clipboard_tracking_consent")));
static FILE_INDEXING_ENABLED: Lazy<AtomicBool> =
    Lazy::new(|| AtomicBool::new(read_flag("file_indexing_enabled")));

fn path(name: &str) -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("quarry")
        .join(name)
}

fn read_flag(name: &str) -> bool {
    fs::read(path(name)).is_ok_and(|contents| contents == b"1")
}

fn write_flag(name: &str, value: bool) -> Result<(), String> {
    let path = path(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| error.to_string())?
    };
    #[cfg(not(unix))]
    let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
    file.write_all(if value { b"1" } else { b"0" })
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn clipboard_enabled() -> bool {
    CLIPBOARD_ENABLED.load(Ordering::Acquire)
}

pub fn set_clipboard_enabled(value: bool) -> Result<(), String> {
    if !value {
        crate::CLIPBOARD_MANAGER.stop_monitoring();
        CLIPBOARD_ENABLED.store(false, Ordering::Release);
    }
    write_flag("clipboard_tracking_consent", value)?;
    CLIPBOARD_ENABLED.store(value, Ordering::Release);
    if value {
        crate::CLIPBOARD_MANAGER.start_monitoring();
    }
    Ok(())
}

pub fn file_indexing_enabled() -> bool {
    FILE_INDEXING_ENABLED.load(Ordering::Acquire)
}

pub fn set_file_indexing_enabled(value: bool) -> Result<(), String> {
    if !value {
        crate::searchers::files::stop_file_index();
        FILE_INDEXING_ENABLED.store(false, Ordering::Release);
    }
    write_flag("file_indexing_enabled", value)?;
    FILE_INDEXING_ENABLED.store(value, Ordering::Release);
    if value {
        crate::searchers::files::start_file_index();
    }
    Ok(())
}

pub fn clear_for_reset() {
    CLIPBOARD_ENABLED.store(false, Ordering::Release);
    FILE_INDEXING_ENABLED.store(false, Ordering::Release);
}
