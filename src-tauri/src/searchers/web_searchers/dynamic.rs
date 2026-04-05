use crate::searchers::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use tauri::AppHandle;

pub struct WebSearcher {
    pub name:         String,
    pub url_template: String,
    pub icon:         Option<String>,
}

impl SearchProvider for WebSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();
        let url = self.url_template.replace("{}", &urlencoding::encode(q));

        let mut item = ResultItem::new(
            format!("{} → '{}'", self.name, q),
            ActionData::OpenUrl { url },
        )
        .description("Open in browser");

        if let Some(icon) = &self.icon {
            item = item.icon(icon.clone());
        }

        SearchResult {
            results: vec![item],
            result_type: ResultType::WebSearch,
        }
    }
}
