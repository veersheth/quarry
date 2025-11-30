use super::super::ListItem; 
use super::super::SearchProvider;

pub struct NixSearcher;

impl SearchProvider for NixSearcher {
    fn search(&self, query: &str) -> Vec<ListItem> {
        let q = query.trim();
        if q.is_empty() {
            return vec![];
        }

        let url = format!("https://search.nixos.org/packages?channel=25.05&query={}", urlencoding::encode(q));

        vec![ListItem {
            name: format!("Search Nix Packages for '{}'", q),
            exec: Some(format!("xdg-open {url}")),
            description: None,
            icon: None,
        }]
    }
}

