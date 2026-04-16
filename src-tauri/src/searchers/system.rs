use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct SystemSearcher;

struct SystemAction {
    name:        &'static str,
    command:     &'static str,
    /// Shown as the description; also searched by fuzzy_filter via the trait.
    description: &'static str,
    icon:        &'static str,
}

const SYSTEM_ACTIONS: &[SystemAction] = &[
    SystemAction {
        name:        "Lock Screen",
        command:     "loginctl lock-session",
        description: "Lock the current session — lock screen secure",
        icon:        "icons/system/lock.png",
    },
    SystemAction {
        name:        "Suspend",
        command:     "systemctl suspend",
        description: "Suspend the system — sleep hibernate",
        icon:        "icons/system/lock.png",
    },
    SystemAction {
        name:        "Hibernate",
        command:     "systemctl hibernate",
        description: "Hibernate the system — save to disk sleep suspend",
        icon:        "icons/system/power.png",
    },
    SystemAction {
        name:        "Shutdown",
        command:     "systemctl poweroff",
        description: "Shut down the system — poweroff power off turn off",
        icon:        "icons/system/power.png",
    },
    SystemAction {
        name:        "Reboot",
        command:     "systemctl reboot",
        description: "Restart the system — reboot reset",
        icon:        "icons/system/reboot.png",
    },
    SystemAction {
        name:        "Log Out",
        command:     "loginctl terminate-user $USER",
        description: "Log out of the current session — logout sign out exit",
        icon:        "icons/system/logout.png",
    },
    SystemAction {
        name:        "Lock Then Suspend",
        command:     "loginctl lock-session && systemctl suspend",
        description: "Lock screen and then suspend — lock sleep",
        icon:        "icons/system/lock.png",
    },
    SystemAction {
        name:        "Emergency Shutdown",
        command:     "systemctl poweroff --force --force",
        description: "Force immediate shutdown — force poweroff kill emergency",
        icon:        "icons/system/power.png",
    },
    SystemAction {
        name:        "Emergency Reboot",
        command:     "systemctl reboot --force --force",
        description: "Force immediate reboot — force restart emergency",
        icon:        "icons/system/reboot.png",
    },
];

impl SearchProvider for SystemSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let candidates: Vec<ResultItem> = SYSTEM_ACTIONS
            .iter()
            .map(|action| {
                ResultItem::new(action.name, vec![ActionData::ShellCommand {
                    command: action.command.to_string(),
                }])
                .description(action.description)
                .icon(action.icon)
            })
            .collect();

        let results = if q.is_empty() {
            candidates
        } else {
            self.fuzzy_filter(candidates, q)
        };

        SearchResult { results, result_type: ResultType::List }
    }
}
