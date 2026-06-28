use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

/// Normalize text for better matching by:
/// - Converting to lowercase
/// - Replacing common separators (-, _, ., :) with spaces
/// - Collapsing multiple spaces into single spaces
/// - Trimming whitespace
pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            '-' | '_' | '.' | ':' | '/' | '\\' => ' ',
            _ => c,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Smart text matching that tries multiple strategies:
/// 1. Exact match (highest score)
/// 2. Normalized exact match
/// 3. Fuzzy match on original text
/// 4. Fuzzy match on normalized text
pub fn smart_match(text: &str, query: &str) -> Option<i64> {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    // 1. Exact match (case-insensitive)
    if text_lower == query_lower {
        return Some(1000);
    }

    // 2. Exact substring match
    if text_lower.contains(&query_lower) {
        return Some(800);
    }

    // 3. Normalized matching - helps with "this item" matching "this-item"
    let normalized_text = normalize_text(text);
    let normalized_query = normalize_text(query);

    if normalized_text == normalized_query {
        return Some(900);
    }

    if normalized_text.contains(&normalized_query) {
        return Some(700);
    }

    // 4. Fuzzy matching on original text
    if let Some(score) = MATCHER.fuzzy_match(&text_lower, &query_lower) {
        if score > 0 {
            return Some(score);
        }
    }

    // 5. Fuzzy matching on normalized text
    if let Some(score) = MATCHER.fuzzy_match(&normalized_text, &normalized_query) {
        if score > 0 {
            return Some(score / 2); // Lower priority than original text fuzzy
        }
    }

    None
}

/// Multi-field smart matching - returns the best score from any field
pub fn smart_match_multi(fields: &[&str], query: &str) -> Option<i64> {
    fields
        .iter()
        .filter_map(|field| smart_match(field, query))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("this-item"), "this item");
        assert_eq!(normalize_text("some_file.txt"), "some file txt");
        assert_eq!(normalize_text("path/to/file"), "path to file");
        assert_eq!(normalize_text("Multi   Spaces"), "multi spaces");
    }

    #[test]
    fn test_smart_match() {
        // Exact match should score highest
        assert!(smart_match("test", "test").unwrap() > 900);

        // Normalized match should work
        assert!(smart_match("this-item", "this item").is_some());
        assert!(smart_match("some_file", "some file").is_some());

        // Fuzzy match should work
        assert!(smart_match("firefox", "firefo").is_some());
        assert!(smart_match("screenshot", "screnshot").is_some());

        // No match should return None
        assert!(smart_match("completely", "different").is_none());
    }
}