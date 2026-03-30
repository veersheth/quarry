use crate::types::ActionData;
use dashmap::DashMap;

pub struct ActionRegistry {
    actions: DashMap<String, ActionData>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: DashMap::new(),
        }
    }

    pub fn register(&self, id: String, action: ActionData) {
        self.actions.insert(id, action);
    }

    pub fn get_action(&self, id: &str) -> Option<ActionData> {
        self.actions.get(id).map(|v| v.clone())
    }
}
