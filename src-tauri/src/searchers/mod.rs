pub mod apps;
// pub mod emojis;
// pub mod lorem;
// pub mod math;
// pub mod processkiller;
// pub mod shell;
// pub mod web_searchers;

use tauri::AppHandle;
use crate::types::SearchResult;

pub trait SearchProvider {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult;
}
