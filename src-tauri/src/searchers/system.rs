use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct SystemSearcher;

struct SystemAction {
    name: &'static str,
    command: &'static str,
    description: &'static str,
    keywords: &'static [&'static str],
    icon: &'static str,
}

const SYSTEM_ACTIONS: &[SystemAction] = &[
    SystemAction {
        name: "Lock Screen",
        command: "loginctl lock-session",
        description: "Lock the current session",
        keywords: &["lock", "screen", "secure"],
        icon: "icons/system/lock.png",
    },
    SystemAction {
        name: "Suspend",
        command: "systemctl suspend",
        description: "Suspend the system (sleep mode)",
        keywords: &["suspend", "sleep", "hibernate"],
        icon: "icons/system/lock.png",
    },
    SystemAction {
        name: "Hibernate",
        command: "systemctl hibernate",
        description: "Hibernate the system (save to disk)",
        keywords: &["hibernate", "sleep", "suspend"],
        icon: "icons/system/power.png",
    },
    SystemAction {
        name: "Shutdown",
        command: "systemctl poweroff",
        description: "Shut down the system",
        keywords: &["shutdown", "poweroff", "power", "off", "turn off"],
        icon: "icons/system/power.png",
    },
    SystemAction {
        name: "Reboot",
        command: "systemctl reboot",
        description: "Restart the system",
        keywords: &["reboot", "restart", "reset"],
        icon: "icons/system/reboot.png",
    },
    SystemAction {
        name: "Log Out",
        command: "loginctl terminate-user $USER",
        description: "Log out of the current session",
        keywords: &["logout", "log out", "exit", "sign out"],
        icon: "icons/system/logout.png",
    },
    SystemAction {
        name: "Lock Then Suspend",
        command: "loginctl lock-session && systemctl suspend",
        description: "Lock screen and then suspend",
        keywords: &["lock suspend", "lock sleep"],
        icon: "icons/system/lock.png",
    },
    SystemAction {
        name: "Emergency Shutdown",
        command: "systemctl poweroff --force --force",
        description: "Force immediate shutdown (use with caution)",
        keywords: &["force shutdown", "emergency", "kill"],
        icon: "icons/system/power.png",
    },
    SystemAction {
        name: "Emergency Reboot",
        command: "systemctl reboot --force --force",
        description: "Force immediate reboot (use with caution)",
        keywords: &["force reboot", "emergency restart"],
        icon: "icons/system/reboot.png",
    },
];

impl SystemSearcher {
    fn matches_query(action: &SystemAction, query: &str) -> bool {
        let q = query.to_lowercase();
        if action.name.to_lowercase().contains(&q) {
            return true;
        }
        action.keywords.iter().any(|kw| kw.contains(&q) || q.contains(kw))
    }
}

impl SearchProvider for SystemSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let results = SYSTEM_ACTIONS
            .iter()
            .filter(|action| q.is_empty() || Self::matches_query(action, q))
            .map(|action| {
                ResultItem::new(action.name, ActionData::ShellCommand {
                    command: action.command.to_string(),
                })
                .description(action.description)
                .icon(action.icon)
            })
            .collect();

        SearchResult {
            results,
            result_type: ResultType::List,
        }
    }
}
