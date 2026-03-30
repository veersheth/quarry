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
                result_type: ResultType::Markdown,
            };
        }

        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);

        let response = match reqwest::blocking::get(url) {
            Ok(resp) => resp,
            Err(_) => return SearchResult {
                results: vec![
                    ResultItem::new(
                        format!("Failed to fetch '{}'", word),
                        ActionData::None,
                    )
                    .description("# Error\n\nCheck your internet connection."),
                ],
                result_type: ResultType::Markdown,
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
                    .description(format!("# Not Found\n\n'{}' was not found in the dictionary.", word)),
                ],
                result_type: ResultType::Markdown,
            },
        };

        let mut results = vec![];

        if let Some(first) = definitions.first() {
            let phonetic_text = first.phonetics.iter()
                .find_map(|p| p.text.clone())
                .unwrap_or_default();

            let mut markdown = format!("# {} {}\n\n", first.word, phonetic_text);

            for meaning in &first.meanings {
                markdown.push_str(&format!("### {}\n\n", meaning.part_of_speech));
                for (i, def) in meaning.definitions.iter().take(3).enumerate() {
                    markdown.push_str(&format!("{}. {}\n\n", i + 1, def.definition));
                    if let Some(example) = &def.example {
                        markdown.push_str(&format!("> *{}*\n\n", example));
                    }
                }
            }

            results.push(
                ResultItem::new(
                    format!("{} {}", first.word, phonetic_text),
                    ActionData::CopyToClipboard { text: first.word.clone() },
                )
                .description(markdown),
            );
        }

        SearchResult {
            results,
            result_type: ResultType::Markdown,
        }
    }
}
