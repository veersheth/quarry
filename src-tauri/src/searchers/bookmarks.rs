use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bookmark {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BookmarksData {
    bookmarks: Vec<Bookmark>,
}

pub struct BookmarksSearcher;

impl BookmarksSearcher {
    fn get_bookmarks_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let config_dir = home.join(".config/quarry");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).ok()?;
        }
        Some(config_dir.join("bookmarks.json"))
    }

    fn load_bookmarks() -> Result<Vec<Bookmark>, String> {
        let path = Self::get_bookmarks_path()
            .ok_or_else(|| "could not determine bookmarks path".to_string())?;

        if !path.exists() {
            let empty_data = BookmarksData { bookmarks: vec![] };
            let json = serde_json::to_string_pretty(&empty_data)
                .map_err(|e| format!("failed to serialize: {}", e))?;
            fs::write(&path, json)
                .map_err(|e| format!("failed to create bookmarks file: {}", e))?;
            return Ok(vec![]);
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read bookmarks: {}", e))?;
        let data: BookmarksData = serde_json::from_str(&contents)
            .map_err(|e| format!("failed to parse bookmarks: {}", e))?;
        Ok(data.bookmarks)
    }

    fn save_bookmarks(bookmarks: Vec<Bookmark>) -> Result<(), String> {
        let path = Self::get_bookmarks_path()
            .ok_or_else(|| "could not determine bookmarks path".to_string())?;
        let data = BookmarksData { bookmarks };
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("failed to serialize: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("failed to write bookmarks: {}", e))?;
        Ok(())
    }

    pub fn add_bookmark(name: String, url: String) -> Result<String, String> {
        let mut bookmarks = Self::load_bookmarks()?;
        if bookmarks.iter().any(|b| b.name.to_lowercase() == name.to_lowercase()) {
            return Err(format!("bookmark '{}' already exists", name));
        }
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url
        };
        bookmarks.push(Bookmark { name: name.clone(), url });
        Self::save_bookmarks(bookmarks)?;
        Ok(format!("added bookmark: {}", name))
    }

    pub fn remove_bookmark(name: &str) -> Result<String, String> {
        let mut bookmarks = Self::load_bookmarks()?;
        let original_len = bookmarks.len();
        bookmarks.retain(|b| b.name.to_lowercase() != name.to_lowercase());
        if bookmarks.len() == original_len {
            return Err(format!("bookmark '{}' not found", name));
        }
        Self::save_bookmarks(bookmarks)?;
        Ok(format!("removed bookmark: {}", name))
    }

    fn is_url(s: &str) -> bool {
        s.contains('.') && (
            s.starts_with("http://") ||
            s.starts_with("https://") ||
            s.contains("://") ||
            s.split('.').count() >= 2
        )
    }
}

impl SearchProvider for BookmarksSearcher {
    fn name(&self) -> String { "bookmarks".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim();

        let bookmarks = match Self::load_bookmarks() {
            Ok(b) => b,
            Err(e) => return SearchResult {
                results: vec![
                    ResultItem::new(format!("error loading bookmarks: {}", e), vec![Action::new("", ActionData::None)])
                        .icon("icons/bookmark.png")
                ],
                result_type: ResultType::List,
                            ..Default::default()
},
        };

        if query.is_empty() {
            if bookmarks.is_empty() {
                return SearchResult {
                    results: vec![
                        ResultItem::new("No bookmarks yet", vec![Action::new("", ActionData::None)])
                            .description("Type a name and URL to add a bookmark")
                            .icon("icons/bookmark.png")
                    ],
                    result_type: ResultType::List,
                                    ..Default::default()
};
            }
            return SearchResult {
                results: bookmarks.iter().map(bookmark_to_item).collect(),
                result_type: ResultType::List,
                            ..Default::default()
};
        }

        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        if parts.len() == 2 && Self::is_url(parts[1]) {
            let name = parts[0].to_string();
            let url = parts[1].to_string();
            return SearchResult {
                results: vec![
                    ResultItem::new(format!("Add bookmark: {}", name), vec![Action::new("Add", ActionData::RunFunction {
                        function_name: "add_bookmark".to_string(),
                        params: vec![name, url.clone()],
                    })])
                    .description(url)
                    .icon("icons/bookmark.png")
                ],
                result_type: ResultType::List,
                            ..Default::default()
};
        }

        let candidates: Vec<ResultItem> = bookmarks.iter().map(bookmark_to_item).collect();
        SearchResult {
            results: self.fuzzy_filter(candidates, query),
            result_type: ResultType::List,
                    ..Default::default()
}
    }
}

fn bookmark_to_item(bookmark: &Bookmark) -> ResultItem {
    ResultItem::new(
        bookmark.name.clone(),
        vec![
            Action::new("Open", ActionData::OpenUrl { url: bookmark.url.clone() }),
            Action::new("Delete", ActionData::RunFunction {
                function_name: "remove_bookmark".to_string(),
                params: vec![bookmark.name.clone()],
            }),
        ],
    )
    .description(bookmark.url.clone())
    .icon("icons/bookmark.png")
}
