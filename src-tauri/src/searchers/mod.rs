pub mod apps;
pub mod emojis; // future
pub mod bookmarks; // future

use crate::searchers::apps::ListItem;

pub trait SearchProvider {
    fn search(&self, query: &str) -> Vec<ListItem>;
}

