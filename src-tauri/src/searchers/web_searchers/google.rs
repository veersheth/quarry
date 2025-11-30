use super::super::ListItem; 
use super::super::SearchProvider;

pub struct GoogleSearcher;

impl SearchProvider for GoogleSearcher {
    fn search(&self, query: &str) -> Vec<ListItem> {
        let q = query.trim();
        if q.is_empty() {
            return vec![];
        }

        let url = format!("https://www.google.com/search?q={}", urlencoding::encode(q));

        vec![ListItem {
            name: format!("Search Google for '{}'", q),
            exec: Some(format!("xdg-open {url}")),
            description: None,
            icon: None,
        }]
    }
}

