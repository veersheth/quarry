use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;
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
            let empty_data = BookmarksData {
                bookmarks: vec![],
            };
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
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim();
        
        let bookmarks = match Self::load_bookmarks() {
            Ok(b) => b,
            Err(e) => {
                return SearchResult {
                    results: vec![ResultItem {
                        name: format!("error loading bookmarks: {}", e),
                        action_id: "error".to_string(),
                        description: None,
                        icon: Some("icons/bookmark.png".to_string()),
                    }],
                    result_type: ResultType::List,
                };
            }
        };
        
        // Empty query - show all bookmarks
        if query.is_empty() {
            if bookmarks.is_empty() {
                return SearchResult {
                    results: vec![ResultItem {
                        name: "No bookmarks yet".to_string(),
                        action_id: "info".to_string(),
                        description: Some("Type a name and URL to add a bookmark".to_string()),
                        icon: Some("icons/bookmark.png".to_string()),
                    }],
                    result_type: ResultType::List,
                };
            }
            
            let results: Vec<ResultItem> = bookmarks
                .iter()
                .map(|bookmark| {
                    let action_id = format!("bookmark_{}", bookmark.name);
                    
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::OpenUrl { url: bookmark.url.clone() },
                        );
                    }
                    
                    ResultItem {
                        name: bookmark.name.clone(),
                        action_id,
                        description: Some(bookmark.url.clone()),
                        icon: Some("icons/bookmark.png".to_string()),
                    }
                })
                .collect();
            
            return SearchResult {
                results,
                result_type: ResultType::List,
            };
        }
        
        // Check if query contains a URL (for adding new bookmarks)
        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        
        if parts.len() == 2 && Self::is_url(parts[1]) {
            // This looks like: "name url.com" - offer to add bookmark
            let name = parts[0].to_string();
            let url = parts[1].to_string();
            
            let action_id = format!("add_bookmark_{}_{}", name, url);
            
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(
                    action_id.clone(),
                    ActionData::RunFunction {
                        function_name: "add_bookmark".to_string(),
                        params: vec![name.clone(), url.clone()],
                    },
                );
            }
            
            return SearchResult {
                results: vec![ResultItem {
                    name: format!("+ Add bookmark: {}", name),
                    action_id,
                    description: Some(url.clone()),
                    icon: Some("icons/bookmark.png".to_string()),
                }],
                result_type: ResultType::List,
            };
        }
        
        // Handle ;d prefix for deletion
        if query.starts_with(";d ") {
            let search_query = query[3..].trim().to_lowercase();
            
            let matching_bookmarks: Vec<&Bookmark> = bookmarks
                .iter()
                .filter(|bookmark| {
                    let name_match = bookmark.name.to_lowercase().contains(&search_query);
                    let url_match = bookmark.url.to_lowercase().contains(&search_query);
                    name_match || url_match
                })
                .collect();
            
            let results: Vec<ResultItem> = matching_bookmarks
                .iter()
                .map(|bookmark| {
                    ResultItem {
                        name: format!("Type: ;d {} (to delete)", bookmark.name),
                        action_id: "info".to_string(),
                        description: Some(bookmark.url.clone()),
                        icon: Some("icons/trash.png".to_string()),
                    }
                })
                .collect();
            
            if results.is_empty() {
                return SearchResult {
                    results: vec![ResultItem {
                        name: "No matching bookmarks found".to_string(),
                        action_id: "info".to_string(),
                        description: Some("Use ;d <name> to delete a bookmark".to_string()),
                        icon: Some("icons/bookmark.png".to_string()),
                    }],
                    result_type: ResultType::List,
                };
            }
            
            return SearchResult {
                results,
                result_type: ResultType::List,
            };
        }
        
        // regular search through existing bookmarks
        let query_lower = query.to_lowercase();
        
        let results: Vec<ResultItem> = bookmarks
            .iter()
            .filter(|bookmark| {
                let name_match = bookmark.name.to_lowercase().contains(&query_lower);
                let url_match = bookmark.url.to_lowercase().contains(&query_lower);
                name_match || url_match
            })
            .map(|bookmark| {
                let action_id = format!("bookmark_{}", bookmark.name);
                
                if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                    registry.register(
                        action_id.clone(),
                        ActionData::OpenUrl { url: bookmark.url.clone() },
                    );
                }
                
                ResultItem {
                    name: bookmark.name.clone(),
                    action_id,
                    description: Some(bookmark.url.clone()),
                    icon: Some("icons/bookmark.png".to_string()),
                }
            })
            .collect();
        
        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
