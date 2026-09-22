use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, SearchResult};

pub struct SystemSearcher;

struct SystemAction {
    name:        &'static str,
    command:     &'static str,
    description: &'static str,
    icon:        &'static str,
    confirm:     bool,
    confirm_title: &'static str,
}

const SYSTEM_ACTIONS: &[SystemAction] = &[
    SystemAction { name: "Lock Screen",        command: "loginctl lock-session",                       description: "Lock the current session - lock screen secure",             icon: "icons/system/lock.png",   confirm: false, confirm_title: "" },
    SystemAction { name: "Suspend",            command: "systemctl suspend",                           description: "Suspend the system - sleep hibernate",                      icon: "icons/system/lock.png",   confirm: false, confirm_title: "" },
    SystemAction { name: "Hibernate",          command: "systemctl hibernate",                         description: "Hibernate the system - save to disk sleep suspend",          icon: "icons/system/power.png",  confirm: true,  confirm_title: "Hibernate?" },
    SystemAction { name: "Shutdown",           command: "systemctl poweroff",                          description: "Shut down the system - poweroff power off turn off",         icon: "icons/system/power.png",  confirm: true,  confirm_title: "Shut down?" },
    SystemAction { name: "Reboot",             command: "systemctl reboot",                            description: "Restart the system - reboot reset",                         icon: "icons/system/reboot.png", confirm: true,  confirm_title: "Reboot?" },
    SystemAction { name: "Log Out",            command: "loginctl terminate-user $USER",               description: "Log out of the current session - logout sign out exit",      icon: "icons/system/logout.png", confirm: true,  confirm_title: "Log out?" },
    SystemAction { name: "Lock Then Suspend",  command: "loginctl lock-session && systemctl suspend",  description: "Lock screen and then suspend - lock sleep",                  icon: "icons/system/lock.png",   confirm: false, confirm_title: "" },
    SystemAction { name: "Emergency Shutdown", command: "systemctl poweroff --force --force",          description: "Force immediate shutdown - force poweroff kill emergency",   icon: "icons/system/power.png",  confirm: true,  confirm_title: "Force shut down?" },
    SystemAction { name: "Emergency Reboot",   command: "systemctl reboot --force --force",            description: "Force immediate reboot - force restart emergency",           icon: "icons/system/reboot.png", confirm: true,  confirm_title: "Force reboot?" },
];

impl SearchProvider for SystemSearcher {
    fn name(&self) -> String { "system".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        let candidates: Vec<ResultItem> = SYSTEM_ACTIONS
            .iter()
            .map(|a| {
                let data = if a.confirm {
                    ActionData::modal(a.confirm_title, &[("Yes", "danger", a.command), ("No", "default", "")])
                } else {
                    ActionData::ShellCommand { command: a.command.into() }
                };
                ResultItem::new(a.name, vec![Action::new("Run", data)])
                    .description(a.description)
                    .icon(a.icon)
            })
            .collect();

        let results = if q.is_empty() { candidates } else { self.fuzzy_filter(candidates, q) };
        SearchResult::list(results)
    }
}
