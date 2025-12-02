use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
use crate::ACTION_REGISTRY;
use serde::Deserialize;

pub struct DictionarySearcher;

#[derive(Deserialize, Debug)]
struct DictionaryResponse {
    word: String,
    meanings: Vec<Meaning>,
    #[serde(default)]
    phonetics: Vec<Phonetic>,
}

#[derive(Deserialize, Debug)]
struct Meaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: String,
    definitions: Vec<Definition>,
}

#[derive(Deserialize, Debug)]
struct Definition {
    definition: String,
    #[serde(default)]
    example: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Phonetic {
    #[serde(default)]
    text: Option<String>,
}

impl SearchProvider for DictionarySearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let word = query.trim();
        
        if word.is_empty() {
            return SearchResult {
                results: vec![],
                result_type: ResultType::List,
            };
        }

        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
        
        let response = match reqwest::blocking::get(url) {
            Ok(resp) => resp,
            Err(_) => {
                return SearchResult {
                    results: vec![ResultItem {
                        name: format!("Failed to fetch definition for '{}'", word),
                        action_id: "dict_error".to_string(),
                        description: Some("Check your internet connection".to_string()),
                        icon: None,
                    }],
                    result_type: ResultType::List,
                };
            }
        };

        let definitions: Vec<DictionaryResponse> = match response.json() {
            Ok(data) => data,
            Err(_) => {
                return SearchResult {
                    results: vec![ResultItem {
                        name: format!("No definition found for '{}'", word),
                        action_id: "dict_not_found".to_string(),
                        description: Some("Word not found in dictionary".to_string()),
                        icon: None,
                    }],
                    result_type: ResultType::List,
                };
            }
        };

        let mut results = vec![];

        if let Some(first) = definitions.first() {
            // Add phonetic pronunciation if available
            if let Some(phonetic) = first.phonetics.first() {
                if let Some(text) = &phonetic.text {
                    let action_id = format!("dict_phonetic_{}", word);
                    
                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::CopyToClipboard {
                                text: text.clone(),
                            },
                        );
                    }

                    results.push(ResultItem {
                        name: format!("{} {}", first.word, text),
                        action_id,
                        description: Some("Copy pronunciation".to_string()),
                        icon: None,
                    });
                }
            }

            // Add definitions grouped by part of speech
            for (idx, meaning) in first.meanings.iter().enumerate() {
                for (def_idx, def) in meaning.definitions.iter().take(3).enumerate() {
                    let action_id = format!("dict_def_{}_{}", idx, def_idx);
                    
                    let description = if let Some(example) = &def.example {
                        format!("Example: {}", example)
                    } else {
                        def.definition.clone()
                    };

                    let full_text = format!(
                        "{} ({}): {}\n{}",
                        first.word,
                        meaning.part_of_speech,
                        def.definition,
                        def.example.as_ref().map(|e| format!("Example: {}", e)).unwrap_or_default()
                    );

                    if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                        registry.register(
                            action_id.clone(),
                            ActionData::CopyToClipboard {
                                text: full_text,
                            },
                        );
                    }

                    results.push(ResultItem {
                        name: format!("[{}] {}", meaning.part_of_speech, def.definition),
                        action_id,
                        description: Some(description),
                        icon: None,
                    });
                }
            }
        }

        SearchResult {
            results,
            result_type: ResultType::Dictionary,
        }
    }
}
