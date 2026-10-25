use crate::types::ActionData;
use dashmap::DashMap;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

const MAX_CACHED_ITEMS: usize = 1_000;

#[derive(Default)]
struct RecentItems {
    items: HashMap<String, Arc<crate::types::ResultItem>>,
    order: VecDeque<String>,
}

pub struct ActionRegistry {
    actions: DashMap<String, ActionData>,
    parents: DashMap<String, Arc<crate::types::ResultItem>>,
    recent_items: Mutex<RecentItems>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: DashMap::new(),
            parents: DashMap::new(),
            recent_items: Mutex::new(RecentItems::default()),
        }
    }

    pub fn register(&self, id: String, action: ActionData) {
        self.actions.insert(id, action);
    }

    pub fn register_item_action(
        &self,
        id: String,
        action: ActionData,
        item: Arc<crate::types::ResultItem>,
    ) {
        self.register(id.clone(), action);
        self.parents.insert(id, item);
    }

    pub fn get_action(&self, id: &str) -> Option<ActionData> {
        self.actions.get(id).map(|v| v.clone())
    }

    pub fn clear(&self) {
        self.actions.clear();
        self.parents.clear();
    }

    pub fn get_parent_item(&self, id: &str) -> Option<Arc<crate::types::ResultItem>> {
        self.parents.get(id).map(|item| Arc::clone(item.value()))
    }

    /// Keep the latest complete result for each action so home recents can
    /// restore the same sibling actions that were shown by its searcher.
    pub fn cache_result_item(&self, item: &crate::types::ResultItem) {
        let Ok(mut cache) = self.recent_items.lock() else {
            return;
        };

        let mut cached_item = item.clone();
        cached_item.thumbnail = None;
        cached_item.thumbnail_key = None;
        cached_item.ocr_text = None;
        let cached_item = Arc::new(cached_item);

        for action in &cached_item.actions {
            let key = format!(
                "{}\0{}",
                cached_item.name.to_lowercase(),
                action.data.stable_id()
            );
            cache.order.retain(|existing| existing != &key);
            cache.order.push_back(key.clone());
            cache.items.insert(key, Arc::clone(&cached_item));
        }

        while cache.items.len() > MAX_CACHED_ITEMS {
            let Some(oldest) = cache.order.pop_front() else {
                break;
            };
            cache.items.remove(&oldest);
        }
    }

    pub fn get_cached_result_item(
        &self,
        name: &str,
        stable_action_id: &str,
    ) -> Option<crate::types::ResultItem> {
        let key = format!("{}\0{}", name.to_lowercase(), stable_action_id);
        self.recent_items
            .lock()
            .ok()?
            .items
            .get(&key)
            .map(|item| item.as_ref().clone())
    }
}
