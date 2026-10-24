use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use once_cell::sync::Lazy;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Emitter};

const DATA_URL: &str = "https://gitlab.com/tdulcet/compact-dictionary/-/raw/main/wiktionary/dictionary-en.json?inline=false";
const UPDATE_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MINIMUM_ENTRY_COUNT: usize = 100_000;

static UPDATE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static RESET_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static UPDATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static LAST_UPDATE_ATTEMPT: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

pub fn begin_reset() -> Result<(), String> {
    let _guard = UPDATE_LOCK
        .lock()
        .map_err(|_| "Dictionary update state is unavailable".to_string())?;
    if UPDATE_IN_PROGRESS.load(Ordering::Relaxed) {
        return Err("Wait for the dictionary download to finish before resetting Quarry".into());
    }
    RESET_IN_PROGRESS.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn end_reset() {
    RESET_IN_PROGRESS.store(false, Ordering::Relaxed);
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DictionaryProgress {
    stage: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    indexed_entries: usize,
    total_entries: usize,
    message: String,
}

#[derive(Deserialize)]
struct DictionaryEntry {
    #[serde(rename = "")]
    word: String,
    #[serde(default)]
    p: Vec<String>,
    #[serde(default)]
    d: Vec<String>,
    #[serde(default)]
    f: Vec<String>,
    #[serde(default)]
    s: Vec<String>,
    #[serde(default)]
    n: Vec<String>,
    #[serde(default)]
    i: Option<String>,
}

pub struct DictionarySearcher;

impl SearchProvider for DictionarySearcher {
    fn name(&self) -> String {
        "dictionary".to_string()
    }

    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let word = query.trim();
        if word.is_empty() {
            return SearchResult {
                result_type: ResultType::Markdown,
                ..Default::default()
            };
        }

        let Some(database) = dictionary_path() else {
            return status_result(
                "Dictionary unavailable",
                "Quarry couldn't find a writable local data directory.",
            );
        };
        maybe_update_dictionary(database.clone(), app.clone());

        if !database.exists() {
            return if UPDATE_IN_PROGRESS.load(Ordering::Relaxed) {
                status_result(
                    "Preparing the local dictionary",
                    "Quarry is downloading and indexing the English dictionary (about 120 MB on first use). It will then work offline and refresh weekly.",
                )
            } else {
                status_result(
                    "Local dictionary setup failed",
                    "Quarry couldn't download the English dictionary. Check your connection; it will retry automatically.",
                )
            };
        }

        match lookup_entry(&database, word) {
            Ok(Some(entry)) => entry_result(entry),
            Ok(None) => status_result(
                &format!("No definition found for '{word}'"),
                &format!(
                    "'{word}' wasn't found in the local English dictionary. Definitions are sourced from Wiktionary and refreshed weekly."
                ),
            ),
            Err(error) => {
                eprintln!("quarry: local dictionary lookup failed: {error}");
                start_dictionary_update(database, app.clone());
                status_result(
                    "Local dictionary unavailable",
                    "Quarry couldn't read its local dictionary. It will try to refresh the data in the background.",
                )
            }
        }
    }
}

fn dictionary_path() -> Option<PathBuf> {
    let mut directory = dirs::data_local_dir()?;
    directory.push("quarry");
    Some(directory.join("dictionary.sqlite"))
}

fn maybe_update_dictionary(database: PathBuf, app: AppHandle) {
    let check_marker = database.with_extension("checked");
    let update_is_due = !database.exists()
        || fs::metadata(&check_marker)
            .or_else(|_| fs::metadata(&database))
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .map_or(true, |age| age >= UPDATE_INTERVAL);
    if !update_is_due {
        return;
    }

    start_dictionary_update(database, app);
}

fn start_dictionary_update(database: PathBuf, app: AppHandle) {
    let Ok(_guard) = UPDATE_LOCK.lock() else {
        return;
    };
    if RESET_IN_PROGRESS.load(Ordering::Relaxed) {
        return;
    }
    let now = Instant::now();
    if let Ok(mut last_attempt) = LAST_UPDATE_ATTEMPT.lock() {
        if last_attempt.is_some_and(|last| now.duration_since(last) < RETRY_INTERVAL) {
            return;
        }
        *last_attempt = Some(now);
    }

    if UPDATE_IN_PROGRESS.swap(true, Ordering::Relaxed) {
        return;
    }

    emit_progress(
        &app,
        "connecting",
        0,
        None,
        0,
        "Connecting to dictionary source…",
    );
    std::thread::spawn(move || {
        if let Err(error) = download_and_index(&database, &app) {
            eprintln!("quarry: dictionary update failed: {error}");
            emit_progress(
                &app,
                "failed",
                0,
                None,
                0,
                &format!("Dictionary update failed: {error}"),
            );
        }
        if let Ok(_guard) = UPDATE_LOCK.lock() {
            UPDATE_IN_PROGRESS.store(false, Ordering::Relaxed);
        }
    });
}

fn download_and_index(
    database: &Path,
    app: &AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = database
        .parent()
        .ok_or("dictionary path has no parent directory")?;
    fs::create_dir_all(directory)?;

    let temporary_database = database.with_extension("sqlite.new");
    let _ = fs::remove_file(&temporary_database);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30 * 60))
        .build()?;
    let etag_path = database.with_extension("etag");
    let mut request = client
        .get(DATA_URL)
        .header(reqwest::header::ACCEPT, "application/octet-stream");
    if database.exists() {
        if let Ok(etag) = fs::read_to_string(&etag_path) {
            request = request.header(reqwest::header::IF_NONE_MATCH, etag.trim());
        }
    }
    let response = request.send()?;
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        fs::write(database.with_extension("checked"), b"checked")?;
        emit_progress(app, "complete", 0, Some(0), 0, "Dictionary is up to date.");
        return Ok(());
    }
    let response = response.error_for_status()?;
    let total_bytes = response.content_length();
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    let mut connection = Connection::open(&temporary_database)?;
    connection.pragma_update(None, "journal_mode", "OFF")?;
    connection.pragma_update(None, "synchronous", "OFF")?;
    connection.execute_batch(
        "CREATE TABLE entries (
             word TEXT PRIMARY KEY COLLATE NOCASE,
             data TEXT NOT NULL
         );
         CREATE TABLE forms (
             form TEXT NOT NULL COLLATE NOCASE,
             word TEXT NOT NULL COLLATE NOCASE,
             PRIMARY KEY (form, word)
         );",
    )?;

    let transaction = connection.transaction()?;
    let mut imported = 0usize;
    let mut downloaded_bytes = 0u64;
    let mut last_reported_bytes = 0u64;
    {
        let mut insert_entry =
            transaction.prepare("INSERT OR REPLACE INTO entries(word, data) VALUES (?1, ?2)")?;
        let mut insert_form =
            transaction.prepare("INSERT OR IGNORE INTO forms(form, word) VALUES (?1, ?2)")?;
        let mut reader = BufReader::new(response);
        let mut line = String::new();

        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            downloaded_bytes = downloaded_bytes.saturating_add(line.len() as u64);

            if let Ok(entry) = serde_json::from_str::<DictionaryEntry>(&line) {
                if !entry.word.is_empty() && !entry.d.is_empty() {
                    insert_entry.execute(params![entry.word, line.trim_end()])?;
                    for form in entry.f {
                        if !form.is_empty() {
                            insert_form.execute(params![form, entry.word])?;
                        }
                    }
                    imported += 1;
                }
            }

            if downloaded_bytes.saturating_sub(last_reported_bytes) >= 1024 * 1024 {
                emit_progress(
                    app,
                    "indexing",
                    downloaded_bytes,
                    total_bytes,
                    imported,
                    "Downloading and indexing the English dictionary…",
                );
                last_reported_bytes = downloaded_bytes;
            }
        }
    }

    emit_progress(
        app,
        "finalizing",
        downloaded_bytes,
        total_bytes,
        imported,
        "Finishing the local dictionary index…",
    );

    if imported < MINIMUM_ENTRY_COUNT {
        return Err(format!("download contained only {imported} dictionary entries").into());
    }

    transaction.commit()?;
    drop(connection);
    fs::rename(temporary_database, database)?;
    if let Some(etag) = etag {
        fs::write(etag_path, etag)?;
    } else {
        let _ = fs::remove_file(etag_path);
    }
    fs::write(database.with_extension("checked"), b"checked")?;
    emit_progress(
        app,
        "complete",
        downloaded_bytes,
        total_bytes,
        imported,
        "Local dictionary is ready.",
    );
    Ok(())
}

fn emit_progress(
    app: &AppHandle,
    stage: &str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    indexed_entries: usize,
    message: &str,
) {
    let progress = DictionaryProgress {
        stage: stage.to_string(),
        downloaded_bytes,
        total_bytes,
        indexed_entries,
        total_entries: 1_064_972,
        message: message.to_string(),
    };
    let _ = app.emit("quarry-dictionary-progress", progress);
}

fn lookup_entry(database: &Path, word: &str) -> rusqlite::Result<Option<DictionaryEntry>> {
    let connection = Connection::open(database)?;
    let direct: Option<String> = connection
        .query_row(
            "SELECT data FROM entries WHERE word = ?1 COLLATE NOCASE",
            [word],
            |row| row.get(0),
        )
        .optional()?;

    let data = match direct {
        Some(data) => Some(data),
        None => connection
            .query_row(
                "SELECT entries.data
                 FROM forms JOIN entries ON entries.word = forms.word
                 WHERE forms.form = ?1 COLLATE NOCASE
                 LIMIT 1",
                [word],
                |row| row.get(0),
            )
            .optional()?,
    };

    Ok(data.and_then(|json| serde_json::from_str(&json).ok()))
}

fn entry_result(entry: DictionaryEntry) -> SearchResult {
    let mut markdown = format!("# {}", escape_markdown(&entry.word));
    if let Some(ipa) = entry.i.as_deref().filter(|ipa| !ipa.is_empty()) {
        markdown.push_str(&format!("  \n/{}/", escape_markdown(ipa)));
    }

    if !entry.p.is_empty() {
        markdown.push_str(&format!("\n\n*{}*\n", entry.p.join(" · ")));
    }

    for (index, definition) in entry.d.iter().take(10).enumerate() {
        markdown.push_str(&format!("\n{}. {}", index + 1, escape_markdown(definition)));
    }

    append_related_words(&mut markdown, "Synonyms", &entry.s);
    append_related_words(&mut markdown, "Antonyms", &entry.n);
    markdown.push_str(&format!(
        "\n\n---\n\n*Source: [Wiktionary](https://en.wiktionary.org/wiki/{}) · CC BY-SA 3.0 / GFDL · weekly data via [Compact Dictionaries](https://github.com/tdulcet/compact-dictionaries).*",
        urlencoding::encode(&entry.word)
    ));

    let item = ResultItem::new(
        entry.word.clone(),
        vec![Action::new(
            "Copy",
            ActionData::CopyToClipboard {
                text: entry.word.clone(),
            },
        )],
    )
    .description(markdown);

    SearchResult {
        results: vec![item],
        result_type: ResultType::Markdown,
        ..Default::default()
    }
}

fn append_related_words(markdown: &mut String, heading: &str, words: &[String]) {
    if words.is_empty() {
        return;
    }
    let words = words
        .iter()
        .take(12)
        .map(|word| escape_markdown(word))
        .collect::<Vec<_>>()
        .join(", ");
    markdown.push_str(&format!("\n\n**{heading}:** {words}"));
}

fn escape_markdown(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn status_result(title: &str, message: &str) -> SearchResult {
    SearchResult {
        results: vec![
            ResultItem::new(title, vec![Action::new("", ActionData::None)]).description(message),
        ],
        result_type: ResultType::Markdown,
        ..Default::default()
    }
}
