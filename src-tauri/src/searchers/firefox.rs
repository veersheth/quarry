use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;
use std::path::PathBuf;

pub struct FirefoxSearcher;

impl FirefoxSearcher {
    fn find_firefox_profile() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        
        // #[cfg(target_os = "linux")]
        let firefox_dir = home.join(".mozilla/firefox");
        
        if !firefox_dir.exists() {
            return None;
        }
        
        let entries = std::fs::read_dir(&firefox_dir).ok()?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()?.to_str()?;
                if name.contains(".default") {
                    return Some(path);
                }
            }
        }
        
        None
    }
    
    fn load_bookmarks() -> Result<Vec<(String, String)>, String> {
        let profile = Self::find_firefox_profile()
            .ok_or_else(|| "Firefox profile not found".to_string())?;
        
        let db_path = profile.join("places.sqlite");
        
        if !db_path.exists() {
            return Err("Firefox places.sqlite not found".to_string());
        }
        
        // avoid locking
        let temp_db = std::env::temp_dir().join("quarry_firefox_places.sqlite");
        std::fs::copy(&db_path, &temp_db)
            .map_err(|e| format!("Failed to copy database: {}", e))?;
        
        let conn = rusqlite::Connection::open(&temp_db)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let mut stmt = conn
            .prepare(
                "SELECT moz_bookmarks.title, moz_places.url 
                 FROM moz_bookmarks 
                 JOIN moz_places ON moz_bookmarks.fk = moz_places.id 
                 WHERE moz_bookmarks.type = 1 
                 AND moz_places.url NOT LIKE 'place:%'
                 AND moz_bookmarks.title IS NOT NULL"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        
        let bookmarks = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?
                ))
            })
            .map_err(|e| format!("Failed to query bookmarks: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect bookmarks: {}", e))?;
        
        let _ = std::fs::remove_file(&temp_db);
        
        Ok(bookmarks)
    }
}

impl SearchProvider for FirefoxSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query_lower = query.trim().to_lowercase();
        
        let bookmarks = match Self::load_bookmarks() {
            Ok(b) => b,
            Err(e) => {
                return SearchResult {
                    results: vec![ResultItem {
                        name: format!("Error loading bookmarks: {}", e),
                        action_id: "error".to_string(),
                        description: None,
                        icon: None,
                    }],
                    result_type: ResultType::List,
                    usage_sorted: true,
                    additional_info: None,
                };
            }
        };
        
        let mut results: Vec<ResultItem> = bookmarks
            .iter()
            .filter(|(title, url)| {
                let title_match = title.to_lowercase().contains(&query_lower);
                let url_match = url.to_lowercase().contains(&query_lower);
                title_match || url_match
            })
            .take(10)
            .map(|(title, url)| {
                let action_id = format!("firefox_{}", url);
                
                if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                    registry.register(
                        action_id.clone(),
                        ActionData::OpenUrl { url: url.clone() },
                    );
                }
                
                ResultItem {
                    name: title.clone(),
                    action_id,
                    description: Some(url.clone()),
                    icon: Some("icons/bookmark.png".to_string()),
                }
            })
            .collect();
        
        SearchResult {
            results,
            result_type: ResultType::List,
            usage_sorted: true,
            additional_info: None,
        }
    }
}
