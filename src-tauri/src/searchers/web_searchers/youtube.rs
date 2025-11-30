use super::super::ListItem; 
use super::super::SearchProvider;

pub struct YouTubeSearcher;

impl SearchProvider for YouTubeSearcher {
    fn search(&self, query: &str) -> Vec<ListItem> {
        let q = query.trim();
        if q.is_empty() {
            return vec![];
        }

        let url = format!("https://www.youtube.com/search?q={}", urlencoding::encode(q));

        vec![ListItem {
            name: format!("Search YouTube for '{}'", q),
            exec: Some(format!("xdg-open {url}")),
            description: None,
            icon: None,
        }]
    }
}

