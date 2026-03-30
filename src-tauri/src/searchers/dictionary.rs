use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ResultItem, ResultType, SearchResult, ActionData};
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
            Err(_) => return SearchResult {
                results: vec![
                    ResultItem::new(
                        format!("Failed to fetch definition for '{}'", word),
                        ActionData::None,
                    )
                    .description("Check your internet connection"),
                ],
                result_type: ResultType::List,
            },
        };

        let definitions: Vec<DictionaryResponse> = match response.json() {
            Ok(data) => data,
            Err(_) => return SearchResult {
                results: vec![
                    ResultItem::new(
                        format!("No definition found for '{}'", word),
                        ActionData::None,
                    )
                    .description("Word not found in dictionary"),
                ],
                result_type: ResultType::List,
            },
        };

        let mut results = vec![];

        if let Some(first) = definitions.first() {
            if let Some(phonetic) = first.phonetics.first() {
                if let Some(text) = &phonetic.text {
                    results.push(
                        ResultItem::new(
                            format!("{} {}", first.word, text),
                            ActionData::CopyToClipboard { text: text.clone() },
                        )
                        .description("Copy pronunciation"),
                    );
                }
            }

            for (_idx, meaning) in first.meanings.iter().enumerate() {
                for (_def_idx, def) in meaning.definitions.iter().take(3).enumerate() {
                    let description = match &def.example {
                        Some(example) => format!("Example: {}", example),
                        None => def.definition.clone(),
                    };

                    let full_text = format!(
                        "{} ({}): {}\n{}",
                        first.word,
                        meaning.part_of_speech,
                        def.definition,
                        def.example.as_ref().map(|e| format!("Example: {}", e)).unwrap_or_default()
                    );

                    results.push(
                        ResultItem::new(
                            format!("[{}] {}", meaning.part_of_speech, def.definition),
                            ActionData::CopyToClipboard { text: full_text },
                        )
                        .description(description),
                    );
                }
            }
        }

        SearchResult {
            results,
            result_type: ResultType::Dictionary,
        }
    }
}
