use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageEntry {
    pub query: String,
    pub action_id: String,
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

    pub fn record_usage(&mut self, query: &str, action_id: &str, name: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.action_id == action_id && e.query.to_lowercase() == query.to_lowercase())
        {
            entry.count += 1;
            entry.last_used = now;
            entry.name = name.to_string();
        } else {
            self.entries.push(UsageEntry {
                query: query.to_string(),
                action_id: action_id.to_string(),
                name: name.to_string(),
                last_used: now,
                count: 1,
            });
        }

        // Store only last 1000 entries
        if self.entries.len() > 1000 {
            self.entries.sort_by_key(|e| e.last_used);
            self.entries.drain(0..self.entries.len() - 1000);
        }

        self.save();
    }

    fn get_boost_scores(&self, query: &str, action_ids: &HashSet<String>) -> HashMap<String, f32> {
        let query_lower = query.to_lowercase();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut scores = HashMap::new();

        for entry in &self.entries {
            if !action_ids.contains(&entry.action_id) {
                continue;
            }

            // Calculate query similarity
            let entry_query = entry.query.to_lowercase();
            let query_match = if query_lower.is_empty() {
                0.5 // Neutral score for empty queries
            } else if entry_query.contains(&query_lower) || query_lower.contains(&entry_query) {
                1.0
            } else if fuzzy_match_lowercase(&entry_query, &query_lower) {
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
            let score = scores.entry(entry.action_id.clone()).or_insert(0.0f32);
            *score = score.max(total_score);
        }

        scores
    }

    fn fuzzy_match(&self, entry_query: &str, user_query: &str) -> bool {
        let entry_lower = entry_query.to_lowercase();
        let user_lower = user_query.to_lowercase();

        fuzzy_match_lowercase(&entry_lower, &user_lower)
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

        // Sort by last_used descending
        relevant.sort_by_key(|entry| std::cmp::Reverse(entry.last_used));
        relevant.truncate(limit);
        relevant
    }
}

pub fn boost_results_by_usage(
    results: Vec<crate::types::ResultItem>,
    query: &str,
    history: &UsageHistory,
) -> Vec<crate::types::ResultItem> {
    let action_ids: HashSet<String> = results
        .iter()
        .filter_map(|result| result.actions.first())
        .map(|action| action.data.stable_id())
        .collect();
    let scores = history.get_boost_scores(query, &action_ids);
    let mut scored: Vec<_> = results
        .into_iter()
        .map(|result| {
            let score = result
                .actions
                .first()
                .and_then(|action| scores.get(&action.data.stable_id()))
                .copied()
                .unwrap_or_default();
            (result, score)
        })
        .collect();
    scored.sort_by(|left, right| right.1.total_cmp(&left.1));
    scored.into_iter().map(|(result, _)| result).collect()
}

fn fuzzy_match_lowercase(entry_query: &str, user_query: &str) -> bool {
    let entry_words: Vec<&str> = entry_query.split_whitespace().collect();
    let user_words: Vec<&str> = user_query.split_whitespace().collect();

    user_words.iter().all(|word| {
        entry_words
            .iter()
            .any(|entry_word| entry_word.contains(word) || word.contains(entry_word))
    })
}

/// Read recent usage entries for a given searcher namespace (prefix on action_ids).
/// Searchers call this to populate their history view without needing direct access
/// to the USAGE_HISTORY global.
pub fn get_recent_entries(query: &str, limit: usize) -> Vec<UsageEntry> {
    if let Ok(history) = crate::USAGE_HISTORY.read() {
        history.get_recent_for_query(query, limit)
    } else {
        vec![]
    }
}
