use super::{SearchProvider, ICON_BOOKMARK};
use tauri::AppHandle;
use crate::types::{Action, ActionData, ResultItem, SearchResult};
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
        let config_dir = dirs::config_dir()?.join("quarry");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).ok()?;
        }
        Some(config_dir.join("bookmarks.json"))
    }

    fn load_bookmarks() -> Result<Vec<Bookmark>, String> {
        let path = Self::get_bookmarks_path()
            .ok_or_else(|| "could not determine bookmarks path".to_string())?;

        if !path.exists() {
            let json = serde_json::to_string_pretty(&BookmarksData { bookmarks: vec![] })
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
        let json = serde_json::to_string_pretty(&BookmarksData { bookmarks })
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
        let url = if url.starts_with("http://") || url.starts_with("https://") {
            url
        } else if Self::is_localhost(&url) {
            format!("http://{}", url)
        } else {
            format!("https://{}", url)
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

    fn is_localhost(s: &str) -> bool {
        let host = s.split_once("://").map(|(_, rest)| rest).unwrap_or(s);
        let host = host.split('/').next().unwrap_or(host);
        let host = host.split(':').next().unwrap_or(host);
        host == "localhost" || host == "127.0.0.1" || host == "::1"
    }
}

impl SearchProvider for BookmarksSearcher {
    fn name(&self) -> String { "bookmarks".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim();

        let bookmarks = match Self::load_bookmarks() {
            Ok(b) => b,
            Err(e) => {
                return SearchResult::list(vec![
                    create_bookmark_item(),
                    ResultItem::new(
                        format!("error loading bookmarks: {}", e),
                        vec![Action::new("", ActionData::None)],
                    )
                    .icon(ICON_BOOKMARK),
                ])
            }
        };

        let mut results = vec![create_bookmark_item()];
        results.extend(self.fuzzy_filter(
            bookmarks.iter().map(bookmark_to_item).collect(),
            query,
        ));
        SearchResult::list(results)
    }
}

fn create_bookmark_item() -> ResultItem {
    ResultItem::new(
        "Create new bookmark",
        vec![Action::new(
            "Create",
            ActionData::RunFunction {
                function_name: "open_bookmark_creator".into(),
                params: Vec::new(),
            },
        )],
    )
    .description("Save a website with a name and URL")
    .icon(ICON_BOOKMARK)
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
    .description(format!("Bookmark ({})", bookmark.url))
    .icon(favicon_url(&bookmark.url))
}

fn favicon_url(url: &str) -> String {
    let host = url
        .split_once("://")
        .map(|(_, remainder)| remainder)
        .unwrap_or(url)
        .split(|character| matches!(character, '/' | '?' | '#'))
        .next()
        .unwrap_or("");
    format!(
        "https://www.google.com/s2/favicons?domain={}&sz=64",
        urlencoding::encode(host)
    )
}
