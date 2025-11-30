use crate::searchers::{ListItem, SearchProvider};

pub struct MathSearcher;

impl SearchProvider for MathSearcher {
    fn search(&self, query: &str) -> Vec<ListItem> {
        let expr = query.trim();

        let value = meval::eval_str(expr);
        match value {
            Ok(result) => vec![ListItem {
                name: format!("{} = {}", expr, result),
                exec: Some(format!("wl-copy {}", result)),
                description: Some("Copy to clipboard".into()),
                icon: None,
            }],
            Err(_) => vec![],
        }
    }
}
