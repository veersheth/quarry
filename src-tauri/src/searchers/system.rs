use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;

pub struct SystemSearcher;

struct SystemAction {
    name: &'static str,
    command: &'static str,
    description: &'static str,
    keywords: &'static [&'static str],
}

const SYSTEM_ACTIONS: &[SystemAction] = &[
    SystemAction {
        name: "Lock Screen",
        command: "loginctl lock-session",
        description: "Lock the current session",
        keywords: &["lock", "screen", "secure"],
    },
    SystemAction {
        name: "Suspend",
        command: "systemctl suspend",
        description: "Suspend the system (sleep mode)",
        keywords: &["suspend", "sleep", "hibernate"],
    },
    SystemAction {
        name: "Hibernate",
        command: "systemctl hibernate",
        description: "Hibernate the system (save to disk)",
        keywords: &["hibernate", "sleep", "suspend"],
    },
    SystemAction {
        name: "Shutdown",
        command: "systemctl poweroff",
        description: "Shut down the system",
        keywords: &["shutdown", "poweroff", "power", "off", "turn off"],
    },
    SystemAction {
        name: "Reboot",
        command: "systemctl reboot",
        description: "Restart the system",
        keywords: &["reboot", "restart", "reset"],
    },
    SystemAction {
        name: "Log Out",
        command: "loginctl terminate-user $USER",
        description: "Log out of the current session",
        keywords: &["logout", "log out", "exit", "sign out"],
    },
    SystemAction {
        name: "Lock Then Suspend",
        command: "loginctl lock-session && systemctl suspend",
        description: "Lock screen and then suspend",
        keywords: &["lock suspend", "lock sleep"],
    },
    SystemAction {
        name: "Emergency Shutdown",
        command: "systemctl poweroff --force --force",
        description: "Force immediate shutdown (use with caution)",
        keywords: &["force shutdown", "emergency", "kill"],
    },
    SystemAction {
        name: "Emergency Reboot",
        command: "systemctl reboot --force --force",
        description: "Force immediate reboot (use with caution)",
        keywords: &["force reboot", "emergency restart"],
    },
];

impl SystemSearcher {
    fn matches_query(action: &SystemAction, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        if action.name.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        for keyword in action.keywords {
            if keyword.contains(&query_lower) || query_lower.contains(keyword) {
                return true;
            }
        }
        
        false
    }
}

impl SearchProvider for SystemSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let query = query.trim();
        
        let matching_actions: Vec<&SystemAction> = if query.is_empty() {
            SYSTEM_ACTIONS.iter().collect()
        } else {
            SYSTEM_ACTIONS
                .iter()
                .filter(|action| Self::matches_query(action, query))
                .collect()
        };

        let results: Vec<ResultItem> = matching_actions
            .iter()
            .map(|action| {
                let action_id = format!("system_{}", action.name.to_lowercase().replace(" ", "_"));
                
                if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                    registry.register(
                        action_id.clone(),
                        ActionData::ShellCommand {
                            command: action.command.to_string(),
                        },
                    );
                }

                ResultItem {
                    name: action.name.to_string(),
                    action_id,
                    description: Some(action.description.to_string()),
                    icon: None,
                }
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
