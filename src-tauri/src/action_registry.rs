use crate::types::ActionData;
use std::collections::HashMap;

pub struct ActionRegistry {
    actions: HashMap<String, ActionData>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: String, action: ActionData) {
        self.actions.insert(id, action);
    }

    pub fn get_action(&self, id: &str) -> Option<ActionData> {
        self.actions.get(id).cloned()
    }
}
