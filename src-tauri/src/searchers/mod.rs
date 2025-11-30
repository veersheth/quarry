pub mod apps;
pub mod emojis; 
pub mod math; 
pub mod lorem; 
pub mod web_searchers;

use crate::types::{SearchResult, ResultItem, ResultType};

pub trait SearchProvider {
    fn search(&self, query: &str) -> SearchResult;
}

