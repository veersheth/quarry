use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;

/// Expand a leading `~/` to the user's home directory.
/// Paths without `~/` are returned as-is wrapped in `Some`.
pub fn expand_tilde(path: &str) -> Option<std::path::PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        Some(dirs::home_dir()?.join(rest))
    } else {
        Some(std::path::PathBuf::from(path))
    }
}

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

/// Tokenize user input while preserving literal hyphenated terms. For other
/// separators, keep the usual forgiving behavior (`watch later` matches
/// `watch-later` and `watch_later`).
pub fn query_tokens(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .flat_map(|part| {
            if part.contains('-') {
                vec![part.to_lowercase()]
            } else {
                normalize_text(part)
                    .split_whitespace()
                    .map(str::to_string)
                    .collect()
            }
        })
        .collect()
}

/// Normalize a whole query for phrase ranking without treating a literal
/// hyphen as interchangeable with whitespace.
pub fn normalize_query(query: &str) -> String {
    query
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '_' | '.' | ':' | '/' | '\\' => ' ',
            _ => c,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Score a query token against an original, unnormalized field. Hyphenated
/// tokens require a literal hyphen in the field; other tokens use the normal
/// separator-insensitive matching rules.
pub fn query_token_score(text: &str, token: &str) -> i64 {
    if !token.contains('-') {
        return token_match_score(&normalize_text(text), &normalize_text(token));
    }

    let token_lower = token.to_lowercase();
    text.split_whitespace()
        .filter(|word| word.contains('-'))
        .map(|word| {
            let word = word.to_lowercase();
            if word == token_lower {
                1_000
            } else if word.contains(&token_lower) {
                700
            } else {
                MATCHER
                    .fuzzy_match(&word, &token_lower)
                    .filter(|score| *score > 0)
                    .map(|score| 250 + score.min(200))
                    .unwrap_or(0)
            }
        })
        .max()
        .unwrap_or(0)
}

/// Smart text matching that tries multiple strategies:
/// 1. Exact match (highest score)
/// 2. Normalized exact match
/// 3. Fuzzy match on original text
/// 4. Fuzzy match on normalized text
pub fn smart_match(text: &str, query: &str) -> Option<i64> {
    let query_lower = query.to_lowercase();

    if query_lower.contains('-') {
        let score = text
            .split_whitespace()
            .filter(|word| word.contains('-'))
            .filter_map(|word| {
                let word = word.to_lowercase();
                if word == query_lower {
                    Some(1_000)
                } else if word.contains(&query_lower) {
                    Some(800)
                } else {
                    MATCHER.fuzzy_match(&word, &query_lower)
                }
            })
            .max();
        return score.filter(|score| *score > 0);
    }

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

/// Score one normalized query token against normalized text.
/// Direct matches always outrank fuzzy matches, and fuzzy matching is disabled
/// for very short tokens to avoid broad, surprising result sets.
pub fn token_match_score(normalized_text: &str, normalized_token: &str) -> i64 {
    if normalized_text.is_empty() || normalized_token.is_empty() {
        return 0;
    }

    let words = normalized_text.split_whitespace();
    if words.clone().any(|word| word == normalized_token) {
        return 1_000;
    }
    if normalized_text.starts_with(normalized_token) {
        return 850;
    }
    if words.clone().any(|word| word.starts_with(normalized_token)) {
        return 800;
    }
    if normalized_text.contains(normalized_token) {
        return 700;
    }

    if normalized_token.chars().count() >= 3 {
        MATCHER
            .fuzzy_match(normalized_text, normalized_token)
            .filter(|score| *score > 0)
            .map(|score| 250 + score.min(200))
            .unwrap_or(0)
    } else {
        0
    }
}

/// Require every query token to match at least one field. This gives natural
/// word-order-independent matching without accepting a result for only part of
/// a multiword query.
pub fn match_all_tokens(fields: &[&str], query: &str) -> Option<i64> {
    let tokens = query_tokens(query);
    if tokens.is_empty() {
        return None;
    }

    let mut total = 0;
    for token in tokens {
        let best = fields
            .iter()
            .map(|field| query_token_score(field, &token))
            .max()
            .unwrap_or(0);
        if best == 0 {
            return None;
        }
        total += best;
    }
    Some(total)
}

/// Multi-field smart matching - returns the best score from any field
#[allow(dead_code)]
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
