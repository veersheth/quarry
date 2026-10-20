use std::collections::HashSet;
use std::process::Command;

use once_cell::sync::Lazy;
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

static FONT_FAMILIES: Lazy<Vec<String>> = Lazy::new(load_font_families);

pub struct FontsSearcher;

fn load_font_families() -> Vec<String> {
    let Ok(output) = Command::new("fc-list")
        .args(["--format=%{family}\\n"])
        .output()
    else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let families = String::from_utf8_lossy(&output.stdout);
    let mut seen = HashSet::new();
    let mut families = families
        .lines()
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .filter(|family| seen.insert(family.to_lowercase()))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    families.sort_unstable_by_key(|family| family.to_lowercase());
    families
}

impl SearchProvider for FontsSearcher {
    fn name(&self) -> String {
        "fonts".to_string()
    }

    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let items = FONT_FAMILIES
            .iter()
            .map(|family| {
                ResultItem::new(
                    family.clone(),
                    vec![Action::new(
                        "Copy Font Name",
                        ActionData::CopyToClipboard {
                            text: family.clone(),
                        },
                    )],
                )
            })
            .collect();

        let mut results = self.fuzzy_filter(items, query.trim());
        results.truncate(100);
        SearchResult {
            results,
            result_type: ResultType::Fonts,
            ..Default::default()
        }
    }
}
