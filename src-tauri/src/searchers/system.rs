use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct SystemSearcher;

struct SystemAction {
    name:          &'static str,
    command:       &'static str,
    description:   &'static str,
    icon:          &'static str,
    confirm:       bool,
    confirm_label: &'static str,
}

const SYSTEM_ACTIONS: &[SystemAction] = &[
    SystemAction {
        name:          "Lock Screen",
        command:       "loginctl lock-session",
        description:   "Lock the current session - lock screen secure",
        icon:          "icons/system/lock.png",
        confirm:       false,
        confirm_label: "",
    },
    SystemAction {
        name:          "Suspend",
        command:       "systemctl suspend",
        description:   "Suspend the system - sleep hibernate",
        icon:          "icons/system/lock.png",
        confirm:       false,
        confirm_label: "",
    },
    SystemAction {
        name:          "Hibernate",
        command:       "systemctl hibernate",
        description:   "Hibernate the system - save to disk sleep suspend",
        icon:          "icons/system/power.png",
        confirm:       true,
        confirm_label: "Hibernate",
    },
    SystemAction {
        name:          "Shutdown",
        command:       "systemctl poweroff",
        description:   "Shut down the system - poweroff power off turn off",
        icon:          "icons/system/power.png",
        confirm:       true,
        confirm_label: "Shut Down",
    },
    SystemAction {
        name:          "Reboot",
        command:       "systemctl reboot",
        description:   "Restart the system - reboot reset",
        icon:          "icons/system/reboot.png",
        confirm:       true,
        confirm_label: "Reboot",
    },
    SystemAction {
        name:          "Log Out",
        command:       "loginctl terminate-user $USER",
        description:   "Log out of the current session - logout sign out exit",
        icon:          "icons/system/logout.png",
        confirm:       true,
        confirm_label: "Log Out",
    },
    SystemAction {
        name:          "Lock Then Suspend",
        command:       "loginctl lock-session && systemctl suspend",
        description:   "Lock screen and then suspend - lock sleep",
        icon:          "icons/system/lock.png",
        confirm:       false,
        confirm_label: "",
    },
    SystemAction {
        name:          "Emergency Shutdown",
        command:       "systemctl poweroff --force --force",
        description:   "Force immediate shutdown - force poweroff kill emergency",
        icon:          "icons/system/power.png",
        confirm:       true,
        confirm_label: "Force Shut Down",
    },
    SystemAction {
        name:          "Emergency Reboot",
        command:       "systemctl reboot --force --force",
        description:   "Force immediate reboot - force restart emergency",
        icon:          "icons/system/reboot.png",
        confirm:       true,
        confirm_label: "Force Reboot",
    },
];

impl SearchProvider for SystemSearcher {
    fn name(&self) -> String { "system".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let mut candidates: Vec<ResultItem> = SYSTEM_ACTIONS
            .iter()
            .map(|action| {
                let data = if action.confirm {
                    ActionData::RunFunction {
                        function_name: "show_modal".into(),
                        params: vec![
                            format!("{}\n\nAre you sure?", action.name),
                            format!("{}|danger|{}", action.confirm_label, action.command),
                            "Cancel|default|".into(),
                        ],
                    }
                } else {
                    ActionData::ShellCommand { command: action.command.into() }
                };
                ResultItem::new(action.name, vec![Action::new("Run", data)])
                    .description(action.description)
                    .icon(action.icon)
            })
            .collect();


        let results = if q.is_empty() {
            candidates
        } else {
            self.fuzzy_filter(candidates, q)
        };

        SearchResult { results, result_type: ResultType::List, ..Default::default() }
    }
}
