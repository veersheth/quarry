use super::SearchProvider;
use crate::types::{Action, ActionData, ResultItem, ResultType, SearchResult};
use chrono::{Timelike, Utc};
use chrono_tz::Tz;
use std::str::FromStr;
use tauri::AppHandle;

pub struct TimeSearcher;

const CITY_TZ: &[(&str, &str, &str)] = &[
    // North America
    ("New York",       "US",  "America/New_York"),
    ("NYC",            "US",  "America/New_York"),
    ("Los Angeles",    "US",  "America/Los_Angeles"),
    ("LA",             "US",  "America/Los_Angeles"),
    ("Chicago",        "US",  "America/Chicago"),
    ("Houston",        "US",  "America/Chicago"),
    ("Dallas",         "US",  "America/Chicago"),
    ("Denver",         "US",  "America/Denver"),
    ("Phoenix",        "US",  "America/Phoenix"),
    ("Seattle",        "US",  "America/Los_Angeles"),
    ("San Francisco",  "US",  "America/Los_Angeles"),
    ("SF",             "US",  "America/Los_Angeles"),
    ("Miami",          "US",  "America/New_York"),
    ("Boston",         "US",  "America/New_York"),
    ("Atlanta",        "US",  "America/New_York"),
    ("Las Vegas",      "US",  "America/Los_Angeles"),
    ("Minneapolis",    "US",  "America/Chicago"),
    ("Detroit",        "US",  "America/Detroit"),
    ("Toronto",        "CA",  "America/Toronto"),
    ("Vancouver",      "CA",  "America/Vancouver"),
    ("Montreal",       "CA",  "America/Toronto"),
    ("Calgary",        "CA",  "America/Edmonton"),
    ("Mexico City",    "MX",  "America/Mexico_City"),
    // South America
    ("São Paulo",      "BR",  "America/Sao_Paulo"),
    ("Sao Paulo",      "BR",  "America/Sao_Paulo"),
    ("Rio de Janeiro", "BR",  "America/Sao_Paulo"),
    ("Rio",            "BR",  "America/Sao_Paulo"),
    ("Buenos Aires",   "AR",  "America/Argentina/Buenos_Aires"),
    ("Santiago",       "CL",  "America/Santiago"),
    ("Bogota",         "CO",  "America/Bogota"),
    ("Lima",           "PE",  "America/Lima"),
    ("Caracas",        "VE",  "America/Caracas"),
    // Europe
    ("London",         "GB",  "Europe/London"),
    ("Dublin",         "IE",  "Europe/Dublin"),
    ("Lisbon",         "PT",  "Europe/Lisbon"),
    ("Madrid",         "ES",  "Europe/Madrid"),
    ("Barcelona",      "ES",  "Europe/Madrid"),
    ("Paris",          "FR",  "Europe/Paris"),
    ("Amsterdam",      "NL",  "Europe/Amsterdam"),
    ("Brussels",       "BE",  "Europe/Brussels"),
    ("Berlin",         "DE",  "Europe/Berlin"),
    ("Frankfurt",      "DE",  "Europe/Berlin"),
    ("Munich",         "DE",  "Europe/Berlin"),
    ("Hamburg",        "DE",  "Europe/Berlin"),
    ("Zurich",         "CH",  "Europe/Zurich"),
    ("Geneva",         "CH",  "Europe/Zurich"),
    ("Vienna",         "AT",  "Europe/Vienna"),
    ("Rome",           "IT",  "Europe/Rome"),
    ("Milan",          "IT",  "Europe/Rome"),
    ("Stockholm",      "SE",  "Europe/Stockholm"),
    ("Oslo",           "NO",  "Europe/Oslo"),
    ("Copenhagen",     "DK",  "Europe/Copenhagen"),
    ("Helsinki",       "FI",  "Europe/Helsinki"),
    ("Warsaw",         "PL",  "Europe/Warsaw"),
    ("Prague",         "CZ",  "Europe/Prague"),
    ("Budapest",       "HU",  "Europe/Budapest"),
    ("Bucharest",      "RO",  "Europe/Bucharest"),
    ("Athens",         "GR",  "Europe/Athens"),
    ("Istanbul",       "TR",  "Europe/Istanbul"),
    ("Kyiv",           "UA",  "Europe/Kyiv"),
    ("Kiev",           "UA",  "Europe/Kyiv"),
    ("Moscow",         "RU",  "Europe/Moscow"),
    ("St Petersburg",  "RU",  "Europe/Moscow"),
    // Africa
    ("Cairo",          "EG",  "Africa/Cairo"),
    ("Lagos",          "NG",  "Africa/Lagos"),
    ("Nairobi",        "KE",  "Africa/Nairobi"),
    ("Johannesburg",   "ZA",  "Africa/Johannesburg"),
    ("Cape Town",      "ZA",  "Africa/Johannesburg"),
    ("Casablanca",     "MA",  "Africa/Casablanca"),
    ("Accra",          "GH",  "Africa/Accra"),
    ("Addis Ababa",    "ET",  "Africa/Addis_Ababa"),
    ("Dar es Salaam",  "TZ",  "Africa/Dar_es_Salaam"),
    ("Tunis",          "TN",  "Africa/Tunis"),
    // Middle East
    ("Dubai",          "AE",  "Asia/Dubai"),
    ("Abu Dhabi",      "AE",  "Asia/Dubai"),
    ("Riyadh",         "SA",  "Asia/Riyadh"),
    ("Jeddah",         "SA",  "Asia/Riyadh"),
    ("Tel Aviv",       "IL",  "Asia/Jerusalem"),
    ("Jerusalem",      "IL",  "Asia/Jerusalem"),
    ("Beirut",         "LB",  "Asia/Beirut"),
    ("Baghdad",        "IQ",  "Asia/Baghdad"),
    ("Tehran",         "IR",  "Asia/Tehran"),
    ("Doha",           "QA",  "Asia/Qatar"),
    ("Kuwait City",    "KW",  "Asia/Kuwait"),
    ("Amman",          "JO",  "Asia/Amman"),
    // South Asia
    ("Mumbai",         "IN",  "Asia/Kolkata"),
    ("Delhi",          "IN",  "Asia/Kolkata"),
    ("New Delhi",      "IN",  "Asia/Kolkata"),
    ("Bangalore",      "IN",  "Asia/Kolkata"),
    ("Bengaluru",      "IN",  "Asia/Kolkata"),
    ("Chennai",        "IN",  "Asia/Kolkata"),
    ("Hyderabad",      "IN",  "Asia/Kolkata"),
    ("Kolkata",        "IN",  "Asia/Kolkata"),
    ("Karachi",        "PK",  "Asia/Karachi"),
    ("Islamabad",      "PK",  "Asia/Karachi"),
    ("Lahore",         "PK",  "Asia/Karachi"),
    ("Dhaka",          "BD",  "Asia/Dhaka"),
    ("Colombo",        "LK",  "Asia/Colombo"),
    ("Kathmandu",      "NP",  "Asia/Kathmandu"),
    // South-East Asia
    ("Bangkok",        "TH",  "Asia/Bangkok"),
    ("Ho Chi Minh",    "VN",  "Asia/Ho_Chi_Minh"),
    ("Saigon",         "VN",  "Asia/Ho_Chi_Minh"),
    ("Hanoi",          "VN",  "Asia/Bangkok"),
    ("Jakarta",        "ID",  "Asia/Jakarta"),
    ("Bali",           "ID",  "Asia/Makassar"),
    ("Kuala Lumpur",   "MY",  "Asia/Kuala_Lumpur"),
    ("KL",             "MY",  "Asia/Kuala_Lumpur"),
    ("Singapore",      "SG",  "Asia/Singapore"),
    ("Manila",         "PH",  "Asia/Manila"),
    ("Yangon",         "MM",  "Asia/Rangoon"),
    ("Phnom Penh",     "KH",  "Asia/Phnom_Penh"),
    // East Asia
    ("Beijing",        "CN",  "Asia/Shanghai"),
    ("Shanghai",       "CN",  "Asia/Shanghai"),
    ("Shenzhen",       "CN",  "Asia/Shanghai"),
    ("Guangzhou",      "CN",  "Asia/Shanghai"),
    ("Hong Kong",      "HK",  "Asia/Hong_Kong"),
    ("HK",             "HK",  "Asia/Hong_Kong"),
    ("Taipei",         "TW",  "Asia/Taipei"),
    ("Seoul",          "KR",  "Asia/Seoul"),
    ("Tokyo",          "JP",  "Asia/Tokyo"),
    ("Osaka",          "JP",  "Asia/Tokyo"),
    ("Ulaanbaatar",    "MN",  "Asia/Ulaanbaatar"),
    // Central Asia
    ("Tashkent",       "UZ",  "Asia/Tashkent"),
    ("Almaty",         "KZ",  "Asia/Almaty"),
    ("Astana",         "KZ",  "Asia/Almaty"),
    // Oceania
    ("Sydney",         "AU",  "Australia/Sydney"),
    ("Melbourne",      "AU",  "Australia/Melbourne"),
    ("Brisbane",       "AU",  "Australia/Brisbane"),
    ("Perth",          "AU",  "Australia/Perth"),
    ("Adelaide",       "AU",  "Australia/Adelaide"),
    ("Auckland",       "NZ",  "Australia/Lord_Howe"),
    ("Wellington",     "NZ",  "Pacific/Auckland"),
    ("Fiji",           "FJ",  "Pacific/Fiji"),
    ("Honolulu",       "US",  "Pacific/Honolulu"),
    ("Hawaii",         "US",  "Pacific/Honolulu"),
    ("Anchorage",      "US",  "America/Anchorage"),
    // Common abbreviations / UTC offsets people type
    ("UTC",            "",    "UTC"),
    ("GMT",            "",    "Etc/GMT"),
];

// ---------------------------------------------------------------------------
// Country name → representative IANA timezone
// Used when the query matches a country rather than a city.
// ---------------------------------------------------------------------------

const COUNTRY_TZ: &[(&str, &str)] = &[
    ("Afghanistan",          "Asia/Kabul"),
    ("Albania",              "Europe/Tirane"),
    ("Algeria",              "Africa/Algiers"),
    ("Argentina",            "America/Argentina/Buenos_Aires"),
    ("Australia",            "Australia/Sydney"),
    ("Austria",              "Europe/Vienna"),
    ("Bangladesh",           "Asia/Dhaka"),
    ("Belgium",              "Europe/Brussels"),
    ("Bolivia",              "America/La_Paz"),
    ("Brazil",               "America/Sao_Paulo"),
    ("Canada",               "America/Toronto"),
    ("Chile",                "America/Santiago"),
    ("China",                "Asia/Shanghai"),
    ("Colombia",             "America/Bogota"),
    ("Cuba",                 "America/Havana"),
    ("Czech Republic",       "Europe/Prague"),
    ("Czechia",              "Europe/Prague"),
    ("Denmark",              "Europe/Copenhagen"),
    ("Ecuador",              "America/Guayaquil"),
    ("Egypt",                "Africa/Cairo"),
    ("Ethiopia",             "Africa/Addis_Ababa"),
    ("Finland",              "Europe/Helsinki"),
    ("France",               "Europe/Paris"),
    ("Germany",              "Europe/Berlin"),
    ("Ghana",                "Africa/Accra"),
    ("Greece",               "Europe/Athens"),
    ("Hungary",              "Europe/Budapest"),
    ("India",                "Asia/Kolkata"),
    ("Indonesia",            "Asia/Jakarta"),
    ("Iran",                 "Asia/Tehran"),
    ("Iraq",                 "Asia/Baghdad"),
    ("Ireland",              "Europe/Dublin"),
    ("Israel",               "Asia/Jerusalem"),
    ("Italy",                "Europe/Rome"),
    ("Japan",                "Asia/Tokyo"),
    ("Jordan",               "Asia/Amman"),
    ("Kazakhstan",           "Asia/Almaty"),
    ("Kenya",                "Africa/Nairobi"),
    ("Kuwait",               "Asia/Kuwait"),
    ("Lebanon",              "Asia/Beirut"),
    ("Libya",                "Africa/Tripoli"),
    ("Malaysia",             "Asia/Kuala_Lumpur"),
    ("Mexico",               "America/Mexico_City"),
    ("Morocco",              "Africa/Casablanca"),
    ("Myanmar",              "Asia/Rangoon"),
    ("Nepal",                "Asia/Kathmandu"),
    ("Netherlands",          "Europe/Amsterdam"),
    ("New Zealand",          "Pacific/Auckland"),
    ("Nigeria",              "Africa/Lagos"),
    ("Norway",               "Europe/Oslo"),
    ("Pakistan",             "Asia/Karachi"),
    ("Peru",                 "America/Lima"),
    ("Philippines",          "Asia/Manila"),
    ("Poland",               "Europe/Warsaw"),
    ("Portugal",             "Europe/Lisbon"),
    ("Qatar",                "Asia/Qatar"),
    ("Romania",              "Europe/Bucharest"),
    ("Russia",               "Europe/Moscow"),
    ("Saudi Arabia",         "Asia/Riyadh"),
    ("Singapore",            "Asia/Singapore"),
    ("South Africa",         "Africa/Johannesburg"),
    ("South Korea",          "Asia/Seoul"),
    ("Spain",                "Europe/Madrid"),
    ("Sri Lanka",            "Asia/Colombo"),
    ("Sweden",               "Europe/Stockholm"),
    ("Switzerland",          "Europe/Zurich"),
    ("Taiwan",               "Asia/Taipei"),
    ("Tanzania",             "Africa/Dar_es_Salaam"),
    ("Thailand",             "Asia/Bangkok"),
    ("Tunisia",              "Africa/Tunis"),
    ("Turkey",               "Europe/Istanbul"),
    ("UAE",                  "Asia/Dubai"),
    ("Ukraine",              "Europe/Kyiv"),
    ("United Arab Emirates", "Asia/Dubai"),
    ("United Kingdom",       "Europe/London"),
    ("UK",                   "Europe/London"),
    ("United States",        "America/New_York"),
    ("USA",                  "America/New_York"),
    ("US",                   "America/New_York"),
    ("Uruguay",              "America/Montevideo"),
    ("Uzbekistan",           "Asia/Tashkent"),
    ("Venezuela",            "America/Caracas"),
    ("Vietnam",              "Asia/Ho_Chi_Minh"),
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct TzMatch {
    label: String,   // display name, e.g. "Tokyo, JP"
    tz: Tz,
}

fn format_time(tz: Tz) -> String {
    let now = Utc::now().with_timezone(&tz);
    now.format("%-I:%M %p").to_string()
}

fn format_datetime(tz: Tz) -> String {
    let now = Utc::now().with_timezone(&tz);
    now.format("%-I:%M %p, %a %d %b").to_string()
}

fn format_tz_label(tz: Tz) -> String {
    let now = Utc::now().with_timezone(&tz);
    now.format("%Z %:z").to_string()
}

/// Generate an analog clock SVG data URI showing the current time in `tz`.
fn clock_icon(tz: Tz) -> String {
    let now = Utc::now().with_timezone(&tz);
    let h = (now.hour() % 12) as f64;
    let m = now.minute() as f64;

    let hour_angle = h * 30.0 + m * 0.5; // 30° per hour + 0.5° per minute
    let min_angle  = m * 6.0;            // 6° per minute

    let (cx, cy) = (12.0f64, 12.0f64);

    let ha = hour_angle.to_radians();
    let (hx, hy) = (cx + 5.5 * ha.sin(), cy - 5.5 * ha.cos());

    let ma = min_angle.to_radians();
    let (mx, my) = (cx + 8.0 * ma.sin(), cy - 8.0 * ma.cos());

    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'>\
          <circle cx='12' cy='12' r='10' fill='none' stroke='rgba(255,255,255,0.85)' stroke-width='1.5'/>\
          <line x1='12' y1='12' x2='{hx:.2}' y2='{hy:.2}' stroke='rgba(255,255,255,0.9)' stroke-width='2.2' stroke-linecap='round'/>\
          <line x1='12' y1='12' x2='{mx:.2}' y2='{my:.2}' stroke='rgba(255,255,255,0.9)' stroke-width='1.5' stroke-linecap='round'/>\
          <circle cx='12' cy='12' r='1.5' fill='rgba(255,255,255,0.9)'/>\
        </svg>",
    );

    format!("data:image/svg+xml,{}", svg.replace('#', "%23"))
}

/// Build a ResultItem for a timezone match.
fn tz_result(m: &TzMatch, is_pinned: bool) -> ResultItem {
    let pin_action = if is_pinned {
        Action::new("Unpin", ActionData::RunFunction {
            function_name: "unpin".into(),
            params: vec!["time".into(), m.label.clone()],
        })
    } else {
        Action::new("Pin", ActionData::RunFunction {
            function_name: "pin".into(),
            params: vec!["time".into(), m.label.clone(), m.label.clone()],
        })
    };

    let mut item = ResultItem::new(
        format_datetime(m.tz),
        vec![
            Action::new("Copy", ActionData::CopyToClipboard { text: format_time(m.tz) }),
            pin_action,
        ],
    )
    .description(format!("{} · {}", m.label, format_tz_label(m.tz)))
    .icon(clock_icon(m.tz));

    if is_pinned {
        item = item.pinned();
    }

    item
}

/// Collect all matches for the given query, in order: exact, prefix, contains.
fn find_matches(q: &str) -> Vec<TzMatch> {
    let q_lower = q.to_lowercase();
    let mut results: Vec<TzMatch> = Vec::new();
    let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();

    let mut push = |label: String, tz_str: &'static str| {
        if seen.contains(tz_str) {
            return;
        }
        if let Ok(tz) = Tz::from_str(tz_str) {
            seen.insert(tz_str);
            results.push(TzMatch { label, tz });
        }
    };

    // Exact name match first (highest priority)
    for &(city, country, tz_str) in CITY_TZ {
        if city.to_lowercase() == q_lower {
            push(if country.is_empty() { city.to_string() } else { format!("{}, {}", city, country) }, tz_str);
        }
    }
    for &(country, tz_str) in COUNTRY_TZ {
        if country.to_lowercase() == q_lower {
            push(country.to_string(), tz_str);
        }
    }

    for &(city, country, tz_str) in CITY_TZ {
        if city.to_lowercase().starts_with(&q_lower) {
            push(if country.is_empty() { city.to_string() } else { format!("{}, {}", city, country) }, tz_str);
        }
    }
    for &(country, tz_str) in COUNTRY_TZ {
        if country.to_lowercase().starts_with(&q_lower) {
            push(country.to_string(), tz_str);
        }
    }

    for &(city, country, tz_str) in CITY_TZ {
        if city.to_lowercase().contains(&q_lower) {
            push(if country.is_empty() { city.to_string() } else { format!("{}, {}", city, country) }, tz_str);
        }
    }
    for &(country, tz_str) in COUNTRY_TZ {
        if country.to_lowercase().contains(&q_lower) {
            push(country.to_string(), tz_str);
        }
    }

    if results.is_empty() {
        if let Ok(tz) = Tz::from_str(q) {
            results.push(TzMatch {
                label: tz.name().to_string(),
                tz,
            });
        }
    }

    results
}

impl SearchProvider for TimeSearcher {
    fn name(&self) -> String { "time".to_string() }
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let q = query.trim();

        if q.is_empty() {
            let pins = crate::PINS.get("time");
            let pinned_labels: std::collections::HashSet<&str> =
                pins.iter().map(|p| p.name.as_str()).collect();

            let mut results: Vec<ResultItem> = pins
                .iter()
                .filter_map(|pin| {
                    let city = pin.payload.split(',').next()?.trim();
                    find_matches(city).into_iter().next().map(|m| tz_result(&m, true))
                })
                .collect();

            let defaults = ["UTC"];
            results.extend(
                defaults.iter()
                    .flat_map(|name| find_matches(name).into_iter().next())
                    .filter(|m| !pinned_labels.contains(m.label.as_str()))
                    .map(|m| tz_result(&m, false)),
            );

            return SearchResult { results, result_type: ResultType::List, ..Default::default() };
        }

        let matches = find_matches(q);
        let results = matches.iter().take(8).map(|m| {
            let is_pinned = crate::PINS.contains("time", &m.label);
            tz_result(m, is_pinned)
        }).collect();

        SearchResult { results, result_type: ResultType::List, ..Default::default() }
    }
}
