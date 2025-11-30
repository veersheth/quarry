pub mod apps;
pub mod emojis; 
pub mod web_searchers;

use crate::searchers::apps::ListItem;

pub trait SearchProvider {
    fn search(&self, query: &str) -> Vec<ListItem>;
}

