// src-tauri/src/usage_tracker.rs
// Add this to your Cargo.toml dependencies:
// dirs = "5.0"
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageEntry {
    pub query: String,
    pub exec: String,
    pub name: String,
    pub last_used: u64,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UsageHistory {
    entries: Vec<UsageEntry>,
}

impl UsageHistory {
    fn get_storage_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("quarry");
        fs::create_dir_all(&path).ok();
        path.push("usage.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::get_storage_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::get_storage_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            fs::write(path, json).ok();
        }
    }

    pub fn record_usage(&mut self, query: &str, exec: &str, name: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // find existing entry or create new one
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.exec == exec && e.query.to_lowercase() == query.to_lowercase())
        {
            entry.count += 1;
            entry.last_used = now;
        } else {
            self.entries.push(UsageEntry {
                query: query.to_string(),
                exec: exec.to_string(),
                name: name.to_string(),
                last_used: now,
                count: 1,
            });
        }

        // store only last 400 queries
        if self.entries.len() > 400 {
            self.entries.sort_by_key(|e| e.last_used);
            self.entries.drain(0..self.entries.len() - 1000);
        }

        self.save();
    }

    pub fn get_boost_score(&self, query: &str, exec: &str, _name: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut best_score = 0.0f32;

        for entry in &self.entries {
            if entry.exec != exec {
                continue;
            }

            // Calculate query similarity (simple contains check, can be improved with fuzzy matching)
            let query_match = if query_lower.is_empty() {
                0.5 // Neutral score for empty queries
            } else if entry.query.to_lowercase().contains(&query_lower) 
                   || query_lower.contains(&entry.query.to_lowercase()) {
                1.0
            } else if self.fuzzy_match(&entry.query, query) {
                0.7
            } else {
                0.0
            };

            if query_match == 0.0 {
                continue;
            }

            // Frequency score (logarithmic to prevent domination)
            let frequency_score = (entry.count as f32).ln_1p() * 0.3;

            // Recency score (decay over 30 days)
            let age_seconds = now.saturating_sub(entry.last_used);
            let age_days = age_seconds as f32 / 86400.0;
            let recency_score = (1.0 / (1.0 + age_days / 30.0)) * 0.5;

            let total_score = query_match * (frequency_score + recency_score);
            best_score = best_score.max(total_score);
        }

        best_score
    }

    fn fuzzy_match(&self, entry_query: &str, user_query: &str) -> bool {
        let entry_lower = entry_query.to_lowercase();
        let user_lower = user_query.to_lowercase();
        
        let entry_words: Vec<&str> = entry_lower.split_whitespace().collect();
        let user_words: Vec<&str> = user_lower.split_whitespace().collect();

        user_words.iter().all(|word| {
            entry_words.iter().any(|entry_word| {
                entry_word.contains(word) || word.contains(entry_word)
            })
        })
    }

    pub fn get_recent_for_query(&self, query: &str, limit: usize) -> Vec<UsageEntry> {
        let query_lower = query.to_lowercase();
        let mut relevant: Vec<_> = self
            .entries
            .iter()
            .filter(|e| {
                query_lower.is_empty()
                    || e.query.to_lowercase().contains(&query_lower)
                    || e.name.to_lowercase().contains(&query_lower)
                    || self.fuzzy_match(&e.query, query)
            })
            .cloned()
            .collect();

        // last_used descending
        relevant.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        relevant.truncate(limit);
        relevant
    }
}

pub fn boost_results_by_usage(
    mut results: Vec<crate::types::ResultItem>,
    query: &str,
    history: &UsageHistory,
) -> Vec<crate::types::ResultItem> {
    // Create a map of exec -> boost score
    let mut scores: HashMap<String, f32> = HashMap::new();
    
    for result in &results {
        if let Some(exec) = &result.exec {
            let score = history.get_boost_score(query, exec, &result.name);
            scores.insert(exec.clone(), score);
        }
    }

    // Sort results by boost score (higher = more relevant based on history)
    results.sort_by(|a, b| {
        let score_a = a.exec.as_ref().and_then(|e| scores.get(e)).unwrap_or(&0.0);
        let score_b = b.exec.as_ref().and_then(|e| scores.get(e)).unwrap_or(&0.0);
        score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}
