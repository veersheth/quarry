pub mod apps;
pub mod emojis;
pub mod lorem;
pub mod math;
pub mod shell;
pub mod dictionary;
pub mod system;
pub mod web_searchers;
pub mod clipboard;

use tauri::AppHandle;
use crate::types::SearchResult;

pub trait SearchProvider {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult;
}
