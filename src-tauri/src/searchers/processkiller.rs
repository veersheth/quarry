use tauri::AppHandle;

use crate::searchers::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult};
use std::process::Command;

pub struct PkillSearcher;

impl PkillSearcher {
    fn get_running_processes(&self, query: &str) -> Vec<(String, String)> {
        let output = Command::new("ps")
            .args(&["-eo", "comm,cmd", "--no-headers"])
            .output();

        match output {
            Ok(output) => {
                let mut process_map: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();

                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Split into comm and full command
                    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
                    if parts.is_empty() {
                        continue;
                    }

                    let comm = parts[0].to_string();
                    let full_cmd = if parts.len() > 1 {
                        parts[1].trim()
                    } else {
                        parts[0]
                    };

                    // Extract a better display name from the full command
                    let display_name = full_cmd
                        .split('/')
                        .last()
                        .unwrap_or(&comm)
                        .split_whitespace()
                        .next()
                        .unwrap_or(&comm)
                        .to_string();

                    // Filter by query
                    if query.is_empty()
                        || comm.to_lowercase().contains(&query.to_lowercase())
                        || display_name.to_lowercase().contains(&query.to_lowercase())
                        || full_cmd.to_lowercase().contains(&query.to_lowercase())
                    {
                        process_map.insert(display_name.clone(), comm);
                    }
                }

                let mut unique_processes: Vec<(String, String)> = process_map.into_iter().collect();

                unique_processes.sort_by(|a, b| a.0.cmp(&b.0));
                unique_processes
            }
            Err(_) => Vec::new(),
        }
    }
}

impl SearchProvider for PkillSearcher {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let query = query.trim();
        let processes = self.get_running_processes(query);

        let results: Vec<ResultItem> = processes
            .into_iter()
            .map(|(display_name, comm)| ResultItem {
                name: format!("Kill {}", display_name),
                exec: Some(format!("pkill -f {}", comm)),
                description: Some(format!("Terminate: {}", comm)),
                icon: None,
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
