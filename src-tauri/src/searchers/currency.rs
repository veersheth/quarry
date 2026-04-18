use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::AppHandle;

// Cache conversion results for 1 hour; key is "amount:FROM:TO"
static CURRENCY_CACHE: Lazy<DashMap<String, (Instant, SearchResult)>> =
    Lazy::new(DashMap::new);
const CACHE_TTL: Duration = Duration::from_secs(3600);

pub struct CurrencySearcher;

#[derive(Deserialize, Debug)]
struct ConversionResponse {
    amount: f64,
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}

#[derive(Deserialize, Debug)]
struct CurrenciesResponse(HashMap<String, String>);

struct ParsedQuery {
    amount: f64,
    from: String,
    to: String,
}

fn parse_query(input: &str) -> Option<ParsedQuery> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    match parts.as_slice() {
        [amount_str, from, to] if to.to_uppercase() != "TO" => {
            let amount = amount_str.replace(',', ".").parse::<f64>().ok()?;
            Some(ParsedQuery {
                amount,
                from: from.to_uppercase(),
                to: to.to_uppercase(),
            })
        }
        [amount_str, from, _, to] => {
            let amount = amount_str.replace(',', ".").parse::<f64>().ok()?;
            Some(ParsedQuery {
                amount,
                from: from.to_uppercase(),
                to: to.to_uppercase(),
            })
        }
        [from, to] if to.to_uppercase() != "TO" => Some(ParsedQuery {
            amount: 1.0,
            from: from.to_uppercase(),
            to: to.to_uppercase(),
        }),
        [from, _, to] => Some(ParsedQuery {
            amount: 1.0,
            from: from.to_uppercase(),
            to: to.to_uppercase(),
        }),
        _ => None,
    }
}

fn fetch_conversion(amount: f64, from: &str, to: &str) -> Result<ConversionResponse, String> {
    let url = format!(
        "https://api.frankfurter.dev/v1/latest?amount={}&from={}&to={}",
        amount, from, to
    );
    let resp = reqwest::blocking::get(&url).map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json::<ConversionResponse>().map_err(|e| e.to_string())
}

fn fetch_all_currencies() -> Result<HashMap<String, String>, String> {
    let resp = reqwest::blocking::get("https://api.frankfurter.dev/v1/currencies")
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json::<HashMap<String, String>>().map_err(|e| e.to_string())
}

fn fmt_amount(n: f64) -> String {
    let s = format!("{:.6}", n);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    let (int_part, dec_part) = if let Some(dot) = s.find('.') {
        (&s[..dot], Some(&s[dot..]))
    } else {
        (s, None)
    };
    let int_formatted = int_part
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(",");
    match dec_part {
        Some(d) => format!("{}{}", int_formatted, d),
        None => int_formatted,
    }
}

impl SearchProvider for CurrencySearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        if q.is_empty() {
            let recent = crate::usage_tracker::get_recent_entries("", 50);
            // Filter to currency results (their names match "X Y = Z W" pattern)
            let history_items: Vec<ResultItem> = recent
                .into_iter()
                .filter(|e| {
                    // Currency result names look like "100 USD = 91.23 EUR"
                    e.name.contains('=') && e.name.split_whitespace().count() >= 4
                })
                .take(6)
                .map(|e| {
                    // The stored text is the result; re-copy the converted amount on action
                    // Extract the converted value (token after '=')
                    let copied_value = e.name
                        .split('=')
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .unwrap_or("")
                        .to_string();
                    ResultItem::new(
                        e.name.clone(),
                        vec![Action::new("Copy", ActionData::CopyToClipboard { text: copied_value })],
                    )
                    .description(format!("recent: used {} time{}", e.count, if e.count == 1 { "" } else { "s" }))
                    .icon("icons/math.png")
                })
                .collect();

            if history_items.is_empty() {
                return SearchResult {
                    results: vec![ResultItem::new(
                        "Currency converter",
                        vec![Action::new("", ActionData::None)],
                    )
                    .description(
                        "# Currency Converter\n\nExamples:\n- `100 USD to EUR`\n- `50 GBP JPY`\n- `USD EUR`",
                    )],
                    result_type: ResultType::Markdown,
                };
            }

            return SearchResult {
                results: history_items,
                result_type: ResultType::List,
            };
        }

        if q.eq_ignore_ascii_case("list") {
            return match fetch_all_currencies() {
                Ok(map) => {
                    let mut lines: Vec<String> = map
                        .into_iter()
                        .map(|(code, name)| format!("- **{}**: {}", code, name))
                        .collect();
                    lines.sort();
                    let markdown = format!("# Supported Currencies\n\n{}", lines.join("\n"));
                    SearchResult {
                        results: vec![ResultItem::new("Supported currencies", vec![Action::new("", ActionData::None)])
                            .description(markdown)],
                        result_type: ResultType::Markdown,
                    }
                }
                Err(e) => error_result(&format!("Could not fetch currency list: {}", e)),
            };
        }

        let parsed = match parse_query(q) {
            Some(p) => p,
            None => {
                return SearchResult {
                    results: vec![ResultItem::new(
                        "Invalid query",
                        vec![Action::new("", ActionData::None)],
                    )
                    .description(
                        "# Currency Converter\n\nTry:\n- `100 USD to EUR`\n- `GBP JPY`\n- `fx list` for all currencies",
                    )],
                    result_type: ResultType::Markdown,
                };
            }
        };

        if parsed.from == parsed.to {
            return error_result("Source and target currencies must differ.");
        }

        let cache_key = format!("{:.6}:{}:{}", parsed.amount, parsed.from, parsed.to);

        // Return cached result if fresh
        if let Some(entry) = CURRENCY_CACHE.get(&cache_key) {
            let (ts, result) = entry.value();
            if ts.elapsed() < CACHE_TTL {
                return result.clone();
            }
        }

        let result = match fetch_conversion(parsed.amount, &parsed.from, &parsed.to) {
            Ok(data) => {
                let converted = data.rates.get(&parsed.to).copied().unwrap_or(0.0);
                let rate = converted / parsed.amount;

                let result_text = format!(
                    "{} {} = {} {}",
                    fmt_amount(parsed.amount),
                    data.base,
                    fmt_amount(converted),
                    parsed.to
                );

                let markdown = format!(
                    "<div align=\"center\">\n\n# `{}`\n\n</div>\n\n**Rate:** 1 {} = {} {}\n\n*Source: Frankfurter / ECB · {}*",
                    result_text,
                    data.base,
                    fmt_amount(rate),
                    parsed.to,
                    data.date,
                );

                SearchResult {
                    results: vec![ResultItem::new(
                        result_text.clone(),
                        vec![Action::new("Copy", ActionData::CopyToClipboard {
                            text: fmt_amount(converted),
                        })],
                    )
                    .description(markdown)
                    .icon("icons/math.png")],
                    result_type: ResultType::Markdown,
                }
            }
            Err(e) => {
                let msg = if e.contains("422") || e.contains("404") || e.contains("error") {
                    format!(
                        "Unknown currency code(s): **{}** or **{}**.\n\nType `fx list` to see all supported codes.",
                        parsed.from, parsed.to
                    )
                } else {
                    format!("API error: {}", e)
                };
                error_result(&msg)
            }
        };

        // Cache successful (non-error) results
        if result.results.first().map_or(false, |r| r.name != "Currency error") {
            CURRENCY_CACHE.insert(cache_key, (Instant::now(), result.clone()));
        }

        result
    }
}

fn error_result(msg: &str) -> SearchResult {
    SearchResult {
        results: vec![ResultItem::new("Currency error", vec![Action::new("", ActionData::None)])
            .description(format!("# Error\n\n{}", msg))],
        result_type: ResultType::Markdown,
    }
}
