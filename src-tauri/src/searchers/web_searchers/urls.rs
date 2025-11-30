use super::super::ListItem; 
use super::super::SearchProvider;

pub struct URLSearcher;

impl SearchProvider for URLSearcher {
    fn search(&self, query: &str) -> Vec<ListItem> {
        let q = query.trim();
        if q.is_empty() {
            return vec![];
        }

        let url = format!("{}", q);

        vec![ListItem {
            name: format!("Open URL: {}", q),
            exec: Some(format!("xdg-open {url}")),
            description: None,
            icon: None,
        }]
    }
}

