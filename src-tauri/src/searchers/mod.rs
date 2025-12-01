pub mod apps;
pub mod emojis;
pub mod lorem;
pub mod math;
pub mod shell;
pub mod processkiller;
pub mod web_searchers;

use crate::types::SearchResult;

pub trait SearchProvider {
    fn search(&self, query: &str) -> SearchResult;
}
