pub mod apps;
pub mod emojis; 
pub mod math; 
pub mod lorem; 
pub mod shell; 
pub mod web_searchers;

use crate::types::{SearchResult};

pub trait SearchProvider {
    fn search(&self, query: &str) -> SearchResult;
}

