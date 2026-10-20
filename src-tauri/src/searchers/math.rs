use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::AppHandle;

mod dates;
mod units;

// ============================================================
// Calculation history
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
struct CalcEntry {
    expr: String,   // the expression as typed
    result: String, // formatted display value
    raw: String,    // raw value for clipboard (unformatted)
    ts: u64,        // unix seconds
}

static CALC_HISTORY: Lazy<Mutex<Vec<CalcEntry>>> =
    Lazy::new(|| Mutex::new(load_history_from_disk()));

fn history_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".local/share"))
        .join("quarry/calc_history.json")
}

fn load_history_from_disk() -> Vec<CalcEntry> {
    let path = history_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_history_to_disk(entries: &[CalcEntry]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string(entries) {
        std::fs::write(&path, json).ok();
    }
}

pub fn push_to_history(expr: &str, result: &str, raw: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut history = CALC_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
    history.retain(|e| e.expr != expr);
    history.insert(
        0,
        CalcEntry {
            expr: expr.to_string(),
            result: result.to_string(),
            raw: raw.to_string(),
            ts,
        },
    );
    history.truncate(100);
    save_history_to_disk(&history);
}

fn relative_time(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let diff = now.saturating_sub(ts);
    match diff {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", diff / 60),
        3600..=86399 => {
            let h = diff / 3600;
            format!("{} hr{} ago", h, if h == 1 { "" } else { "s" })
        }
        d => {
            let d = d / 86400;
            format!("{} day{} ago", d, if d == 1 { "" } else { "s" })
        }
    }
}

// ============================================================
// Searcher
// ============================================================

pub struct MathSearcher;

impl SearchProvider for MathSearcher {
    fn name(&self) -> String {
        "math".to_string()
    }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let expr = query.trim();

        // Empty query → show history
        if expr.is_empty() {
            let history = CALC_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
            if history.is_empty() {
                return SearchResult::list(vec![ResultItem::new("No history yet", vec![])
                    .description("Calculations will appear here")
                    .icon(super::ICON_MATH)]);
            }
            let results = history
                .iter()
                .map(|e| {
                    ResultItem::new(
                        format!("{} = {}", e.expr, e.result),
                        vec![Action::new(
                            "Copy",
                            ActionData::CopyToClipboard {
                                text: e.raw.clone(),
                            },
                        )],
                    )
                    .description(relative_time(e.ts))
                    .icon(super::ICON_MATH)
                })
                .collect();
            return SearchResult::list(results);
        }

        let mut results = Vec::new();

        if let Some(items) = units::try_unit_conversion(expr) {
            results.extend(items);
        } else if let Some(items) = dates::try_date_calc(expr) {
            results.extend(items);
        } else if let Some(item) = try_percentage(expr) {
            results.push(item);
        } else if let Some(item) = try_math(expr) {
            results.push(item);
        }

        SearchResult {
            results,
            result_type: ResultType::Math,
            ..Default::default()
        }
    }
}

// ============================================================
// Number formatting
// ============================================================

pub(super) fn format_num(n: f64) -> String {
    if n.is_nan() {
        return "NaN".into();
    }
    if n.is_infinite() {
        return if n > 0.0 { "∞".into() } else { "-∞".into() };
    }

    let abs = n.abs();

    if abs != 0.0 && (abs >= 1e15 || abs < 1e-9) {
        let s = format!("{:.6e}", n);
        if let Some(e_pos) = s.find('e') {
            let mantissa = s[..e_pos].trim_end_matches('0').trim_end_matches('.');
            return format!("{}{}", mantissa, &s[e_pos..]);
        }
        return s;
    }

    if n.fract() == 0.0 {
        return commify_int(n as i64);
    }

    let precision = if abs >= 1000.0 {
        3
    } else if abs >= 1.0 {
        6
    } else {
        9
    };
    let s = format!("{:.prec$}", n, prec = precision);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if let Some(dot) = s.find('.') {
        let (int_s, frac_s) = s.split_at(dot);
        format!("{}{}", commify_str(int_s), frac_s)
    } else {
        commify_str(s)
    }
}

fn commify_int(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    let abs_str = n.unsigned_abs().to_string();
    format!("{}{}", sign, commify_str(&abs_str))
}

fn commify_str(s: &str) -> String {
    let (sign, digits) = if let Some(digits) = s.strip_prefix('-') {
        ("-", digits)
    } else {
        ("", s)
    };
    let mut result = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    format!("{}{}", sign, result)
}

// ============================================================
// Result builder
// ============================================================

pub(super) fn make_result(name: String, copy: String) -> ResultItem {
    ResultItem::new(
        name,
        vec![Action::new(
            "Copy",
            ActionData::CopyToClipboard { text: copy },
        )],
    )
    .icon(super::ICON_MATH)
}

// ============================================================
// Date calculations
// ============================================================

// ============================================================
// Math expression evaluation
// ============================================================

fn preprocess_math(expr: &str) -> String {
    static RE_FACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:\.\d+)?)!").unwrap());
    static RE_MOD: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bmod\b").unwrap());

    let s = RE_FACT.replace_all(expr, "fact($1)");
    RE_MOD.replace_all(&s, "%").into_owned()
}

fn try_math(expr: &str) -> Option<ResultItem> {
    if expr.is_empty() {
        return None;
    }
    let preprocessed = preprocess_math(expr);

    // Custom namespace: fasteval calls lookup() before its built-ins, so returning
    // None falls through to fasteval's own sin/cos/sqrt/log/pi/e/^ etc.
    struct MathNS;
    impl fasteval::EvalNamespace for MathNS {
        fn lookup(&mut self, name: &str, args: Vec<f64>, _keybuf: &mut String) -> Option<f64> {
            match (name, args.as_slice()) {
                // Extra constants
                ("tau", []) => Some(std::f64::consts::TAU),
                ("phi", []) => Some(1.618_033_988_749_895_f64),
                // 1-arg extras not in fasteval's built-ins
                ("fact", [x]) => {
                    let x = *x;
                    if x < 0.0 || x.fract() != 0.0 || x > 20.0 {
                        return Some(f64::NAN);
                    }
                    Some((1..=(x as u64)).map(|i| i as f64).product())
                }
                ("cbrt", [x]) => Some(x.cbrt()),
                ("deg", [x]) => Some(x.to_degrees()),
                ("rad", [x]) => Some(x.to_radians()),
                // 2-arg log(base, x) — 1-arg log(x) falls through to fasteval's log10
                ("log", [base, x]) => Some(x.log(*base)),
                // Combinatorics
                ("nPr", [n, r]) => {
                    let (n, r) = (*n, *r);
                    if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
                        return Some(f64::NAN);
                    }
                    let (n, r) = (n as u64, r as u64);
                    Some(((n - r + 1)..=n).map(|i| i as f64).product())
                }
                ("nCr", [n, r]) => {
                    let (n, r) = (*n, *r);
                    if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
                        return Some(f64::NAN);
                    }
                    let (n, r) = (n as u64, r as u64);
                    let r = r.min(n - r);
                    let mut res = 1.0f64;
                    for i in 0..r {
                        res = res * (n - i) as f64 / (i + 1) as f64;
                    }
                    Some(res)
                }
                ("gcd", [a, b]) => {
                    let (mut a, mut b) = (a.abs() as u64, b.abs() as u64);
                    while b != 0 {
                        let t = b;
                        b = a % b;
                        a = t;
                    }
                    Some(a as f64)
                }
                ("lcm", [a, b]) => {
                    let (au, bu) = (a.abs() as u64, b.abs() as u64);
                    if au == 0 || bu == 0 {
                        return Some(0.0);
                    }
                    let (mut g, mut gb) = (au, bu);
                    while gb != 0 {
                        let t = gb;
                        gb = g % gb;
                        g = t;
                    }
                    Some((au / g * bu) as f64)
                }
                _ => None,
            }
        }
    }

    match fasteval::ez_eval(&preprocessed, &mut MathNS) {
        Ok(val) => {
            let formatted = format_num(val);
            let raw = val.to_string();
            let name = format!("{} = {}", expr, formatted);
            // params: [raw_value, expr, formatted_display]
            // History is saved in the executor when the user actually copies.
            let action = crate::types::Action::new(
                "Copy",
                crate::types::ActionData::RunFunction {
                    function_name: "copy_calc".into(),
                    params: vec![raw, expr.to_string(), formatted],
                },
            );
            let mut item = ResultItem::new(name, vec![action]);
            item = item.icon(super::ICON_MATH);
            Some(item)
        }
        Err(_) => None,
    }
}

// ============================================================
// Percentage calculations
// ============================================================

fn try_percentage(expr: &str) -> Option<ResultItem> {
    static RE_OF: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^([\d.]+)%\s*of\s*([\d.,]+)$").unwrap());
    static RE_OFF: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^([\d.]+)%\s*off\s*([\d.,]+)$").unwrap());
    static RE_WHAT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^([\d.,]+)\s+is\s+what\s*%\s*of\s*([\d.,]+)$").unwrap());
    static RE_ADD: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^([\d.,]+)\s*\+\s*([\d.]+)%$").unwrap());

    if let Some(caps) = RE_OF.captures(expr) {
        let pct: f64 = caps[1].parse().ok()?;
        let total: f64 = caps[2].replace(',', "").parse().ok()?;
        let val = total * pct / 100.0;
        let name = format!("{}% of {} = {}", pct, format_num(total), format_num(val));
        return Some(make_result(name, format_num(val)));
    }

    if let Some(caps) = RE_OFF.captures(expr) {
        let pct: f64 = caps[1].parse().ok()?;
        let total: f64 = caps[2].replace(',', "").parse().ok()?;
        let discount = total * pct / 100.0;
        let val = total - discount;
        let name = format!(
            "{}% off {} = {} (save {})",
            pct,
            format_num(total),
            format_num(val),
            format_num(discount)
        );
        return Some(make_result(name, format_num(val)));
    }

    if let Some(caps) = RE_WHAT.captures(expr) {
        let part: f64 = caps[1].replace(',', "").parse().ok()?;
        let total: f64 = caps[2].replace(',', "").parse().ok()?;
        if total == 0.0 {
            return None;
        }
        let pct = part / total * 100.0;
        let name = format!(
            "{} is {}% of {}",
            format_num(part),
            format_num(pct),
            format_num(total)
        );
        return Some(make_result(name, format!("{}%", format_num(pct))));
    }

    if let Some(caps) = RE_ADD.captures(expr) {
        let base: f64 = caps[1].replace(',', "").parse().ok()?;
        let pct: f64 = caps[2].parse().ok()?;
        let val = base * (1.0 + pct / 100.0);
        let added = base * pct / 100.0;
        let name = format!(
            "{} + {}% = {} (+ {})",
            format_num(base),
            pct,
            format_num(val),
            format_num(added)
        );
        return Some(make_result(name, format_num(val)));
    }

    None
}
