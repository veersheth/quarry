use chrono::{Local, TimeZone};
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};

pub struct TimerEntry {
    pub id:           u64,
    pub label:        String,
    pub expires_at:   u64,
    pub duration_secs: u64,
    pub cancelled:    Arc<AtomicBool>,
}

pub static ACTIVE_TIMERS: Lazy<Mutex<Vec<TimerEntry>>> = Lazy::new(|| Mutex::new(vec![]));

pub fn add_timer(id: u64, label: String, expires_at: u64, duration_secs: u64, cancelled: Arc<AtomicBool>) {
    if let Ok(mut t) = ACTIVE_TIMERS.lock() {
        t.push(TimerEntry { id, label, expires_at, duration_secs, cancelled });
    }
}

pub fn remove_timer(id: u64) {
    if let Ok(mut t) = ACTIVE_TIMERS.lock() {
        if let Some(e) = t.iter().find(|e| e.id == id) {
            e.cancelled.store(true, Ordering::Relaxed);
        }
        t.retain(|e| e.id != id);
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut parts = vec![];
    if h > 0 { parts.push(format!("{}h", h)); }
    if m > 0 { parts.push(format!("{}m", m)); }
    if s > 0 || parts.is_empty() { parts.push(format!("{}s", s)); }
    parts.join(" ")
}

fn format_ring_time(unix_secs: u64) -> String {
    Local
        .timestamp_opt(unix_secs as i64, 0)
        .single()
        .map(|dt| dt.format("%-I:%M %p").to_string())
        .unwrap_or_else(|| "??:??".to_string())
}

pub fn parse_timer_query(input: &str) -> Option<(u64, String)> {
    let input = input.trim();
    if input.is_empty() { return None; }

    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut total = 0u64;
    let mut parsed = false;

    loop {
        while i < bytes.len() && bytes[i] == b' ' { i += 1; }
        let num_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
        if i == num_start { break; }

        let n: u64 = input[num_start..i].parse().ok()?;
        while i < bytes.len() && bytes[i] == b' ' { i += 1; }

        if i >= bytes.len() {
            total += n * 60;
            parsed = true;
            break;
        }

        match bytes[i].to_ascii_lowercase() {
            b'h' => { total += n * 3600; while i < bytes.len() && bytes[i].is_ascii_alphabetic() { i += 1; } }
            b'm' => { total += n * 60;   while i < bytes.len() && bytes[i].is_ascii_alphabetic() { i += 1; } }
            b's' => { total += n;        while i < bytes.len() && bytes[i].is_ascii_alphabetic() { i += 1; } }
            _ => {
                total += n * 60;
                parsed = true;
                break;
            }
        }
        parsed = true;
    }

    if !parsed || total == 0 { return None; }
    let label = input[i..].trim().to_string();
    Some((total, label))
}

pub struct TimerSearcher;

impl SearchProvider for TimerSearcher {
    fn name(&self) -> String { "timer".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let mut results: Vec<ResultItem> = vec![];

        if let Some((secs, label)) = parse_timer_query(query) {
            let display = if label.is_empty() {
                format_duration(secs)
            } else {
                format!("{} ({})", label, format_duration(secs))
            };

            let mut params = vec![secs.to_string()];
            if !label.is_empty() { params.push(label.clone()); }

            let ring_at = format_ring_time(now_secs() + secs);
            results.push(
                ResultItem::new(
                    format!("Start: {}", display),
                    vec![Action::new("Start", ActionData::RunFunction {
                        function_name: "start_timer".into(),
                        params,
                    })],
                )
                .description(format!("Alert after {} · rings at {}", format_duration(secs), ring_at)),
            );
        }

        if let Ok(timers) = ACTIVE_TIMERS.lock() {
            for t in timers.iter() {
                let now = now_secs();
                let rem = t.expires_at.saturating_sub(now);
                let ring_at = format_ring_time(t.expires_at);
                let label = if t.label.is_empty() { "Timer".to_string() } else { t.label.clone() };
                results.push(
                    ResultItem::new(
                        format!("{}, {} left", label, format_duration(rem)),
                        vec![Action::new("Cancel", ActionData::RunFunction {
                            function_name: "cancel_timer".into(),
                            params: vec![t.id.to_string()],
                        })],
                    )
                    .description(format!("Rings at {} · {} remaining", ring_at, format_duration(rem))),
                );
            }
        }

        if results.is_empty() {
            results.push(
                ResultItem::new(
                    "Start a timer",
                    vec![],
                )
                .description("timer 5 min, or similar"),
            );
        }

        SearchResult { results, result_type: ResultType::List, ..Default::default() }
    }
}
