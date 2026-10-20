use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::{Action, ActionData, ResultItem};

fn make_date_result(name: String, copy: String) -> ResultItem {
    ResultItem::new(
        name,
        vec![Action::new(
            "Copy",
            ActionData::CopyToClipboard { text: copy },
        )],
    )
    .icon("icons/calendar.png")
}

/// Parse a variety of date formats into a NaiveDate.
/// Supports: YYYY-MM-DD, DD/MM/YYYY, MM/DD/YYYY (US), "Jan 5 2025", "5 Jan 2025",
/// "January 5 2025", "5 January 2025", "next friday", "last monday", "tomorrow", "yesterday"
fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    let today = Local::now().date_naive();

    // ── Relative words ──────────────────────────────────
    match s.to_lowercase().as_str() {
        "today" => return Some(today),
        "tomorrow" => return Some(today + Duration::days(1)),
        "yesterday" => return Some(today - Duration::days(1)),
        _ => {}
    }

    // ── next/last <weekday> ──────────────────────────────
    static RE_NEXT_LAST: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^(next|last)\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday|mon|tue|wed|thu|fri|sat|sun)$").unwrap()
    });
    if let Some(caps) = RE_NEXT_LAST.captures(s) {
        let direction = caps[1].to_lowercase();
        let target = parse_weekday(&caps[2])?;
        let current = today.weekday();
        let current_num = weekday_num(current);
        let target_num = weekday_num(target);
        let delta = if direction == "next" {
            let d = (target_num as i64 - current_num as i64).rem_euclid(7);
            if d == 0 {
                7
            } else {
                d
            }
        } else {
            let d = (current_num as i64 - target_num as i64).rem_euclid(7);
            if d == 0 {
                7
            } else {
                -d
            }
        };
        return Some(today + Duration::days(delta));
    }

    // ── this <weekday> ───────────────────────────────────
    static RE_THIS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^this\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday|mon|tue|wed|thu|fri|sat|sun)$").unwrap()
    });
    if let Some(caps) = RE_THIS.captures(s) {
        let target = parse_weekday(&caps[1])?;
        let current = today.weekday();
        let current_num = weekday_num(current);
        let target_num = weekday_num(target);
        let d = (target_num as i64 - current_num as i64).rem_euclid(7);
        return Some(today + Duration::days(d));
    }

    // ── YYYY-MM-DD ───────────────────────────────────────
    static RE_ISO: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(\d{4})-(\d{1,2})-(\d{1,2})$").unwrap());
    if let Some(caps) = RE_ISO.captures(s) {
        let y: i32 = caps[1].parse().ok()?;
        let m: u32 = caps[2].parse().ok()?;
        let d: u32 = caps[3].parse().ok()?;
        return NaiveDate::from_ymd_opt(y, m, d);
    }

    // ── DD/MM/YYYY or MM/DD/YYYY ─────────────────────────
    // Heuristic: if first number > 12 it must be DD, else try DD/MM first.
    static RE_SLASH: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(\d{1,2})[/\-.](\d{1,2})[/\-.](\d{2,4})$").unwrap());
    if let Some(caps) = RE_SLASH.captures(s) {
        let a: u32 = caps[1].parse().ok()?;
        let b: u32 = caps[2].parse().ok()?;
        let mut y: i32 = caps[3].parse().ok()?;
        if y < 100 {
            y += 2000;
        }
        // if a > 12 → must be DD/MM
        if a > 12 {
            return NaiveDate::from_ymd_opt(y, b, a);
        }
        // else try DD/MM then MM/DD
        return NaiveDate::from_ymd_opt(y, b, a).or_else(|| NaiveDate::from_ymd_opt(y, a, b));
    }

    // ── "Jan 5 2025" / "5 Jan 2025" / "January 5, 2025" etc. ──
    static RE_MONTH_NAME: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^(\d{1,2})\s+([a-z]+)[\s,]+(\d{2,4})$|^([a-z]+)\s+(\d{1,2})[\s,]+(\d{2,4})$",
        )
        .unwrap()
    });
    if let Some(caps) = RE_MONTH_NAME.captures(s) {
        // Two sub-patterns; one will have groups 1-3, other 4-6
        let (day, month_str, year_str) = if caps.get(1).is_some() {
            (&caps[1], &caps[2], &caps[3])
        } else {
            (&caps[5], &caps[4], &caps[6])
        };
        let d: u32 = day.parse().ok()?;
        let m = parse_month_name(month_str)?;
        let mut y: i32 = year_str.parse().ok()?;
        if y < 100 {
            y += 2000;
        }
        return NaiveDate::from_ymd_opt(y, m, d);
    }

    // ── "Jan 5" / "5 Jan" (assume current or next year) ──
    static RE_MONTH_NAME_NO_YEAR: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^(\d{1,2})\s+([a-z]+)$|^([a-z]+)\s+(\d{1,2})$").unwrap());
    if let Some(caps) = RE_MONTH_NAME_NO_YEAR.captures(s) {
        let (day, month_str) = if caps.get(1).is_some() {
            (&caps[1], &caps[2])
        } else {
            (&caps[4], &caps[3])
        };
        let d: u32 = day.parse().ok()?;
        let m = parse_month_name(month_str)?;
        let this_year = today.year();
        // Pick current year; if that date has passed, bump to next year
        let candidate = NaiveDate::from_ymd_opt(this_year, m, d)?;
        return Some(if candidate < today {
            NaiveDate::from_ymd_opt(this_year + 1, m, d)?
        } else {
            candidate
        });
    }

    None
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

fn weekday_num(w: Weekday) -> u32 {
    match w {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

fn parse_month_name(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "sept" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn weekday_name(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

fn format_date(d: NaiveDate) -> String {
    let month = match d.month() {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    };
    format!("{} {} {}", d.day(), month, d.year())
}

/// Break days into a human-readable "X years, Y months, Z days" string.
fn humanize_duration(days: i64) -> String {
    let abs = days.abs();
    let years = abs / 365;
    let months = (abs % 365) / 30;
    let rem = abs % 30;

    let mut parts = Vec::new();
    if years > 0 {
        parts.push(format!(
            "{} year{}",
            years,
            if years == 1 { "" } else { "s" }
        ));
    }
    if months > 0 {
        parts.push(format!(
            "{} month{}",
            months,
            if months == 1 { "" } else { "s" }
        ));
    }
    if rem > 0 || parts.is_empty() {
        parts.push(format!("{} day{}", rem, if rem == 1 { "" } else { "s" }));
    }
    parts.join(", ")
}

pub(super) fn try_date_calc(query: &str) -> Option<Vec<ResultItem>> {
    let q = query.trim();
    let today = Local::now().date_naive();
    let mut results: Vec<ResultItem> = Vec::new();

    // ── "today" / "now" - show date info ────────────────────
    static RE_TODAY: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^(today|now)$").unwrap());
    if RE_TODAY.is_match(q) {
        let iso = today.format("%Y-%m-%d").to_string();
        let human = format_date(today);
        let week = today.iso_week().week();
        let day_of_year = today.ordinal();
        let wd = weekday_name(today.weekday());
        results.push(make_date_result(
            format!(
                "{} - {} (week {}, day {} of year)",
                human, wd, week, day_of_year
            ),
            iso.clone(),
        ));
        results.push(make_date_result(format!("ISO: {}", iso), iso));
        return if results.is_empty() {
            None
        } else {
            Some(results)
        };
    }

    // ── "days until <date>" / "days until christmas" / "days till ..." ────────
    static RE_UNTIL: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^days?\s+(until|till)\s+(.+)$").unwrap());
    // ── "days since <date>" / "days from <date>" ────────────
    static RE_SINCE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^days?\s+(?:since|from)\s+(.+)$").unwrap());
    // ── "<date> to <date>" / "<date> - <date>" ──────────────
    static RE_BETWEEN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^(.+?)\s+(?:to|-)\s+(.+)$").unwrap());
    // ── "add <n> days to <date>" ─────────────────────────────
    static RE_ADD_DAYS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^add\s+(\d+)\s+days?\s+to\s+(.+)$").unwrap());
    // ── "subtract <n> days from <date>" ─────────────────────
    static RE_SUB_DAYS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^subtract\s+(\d+)\s+days?\s+from\s+(.+)$").unwrap());
    // ── "<date> + <n> days" / "<date> - <n> days" ───────────
    static RE_DATE_ARITH: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^(.+?)\s*([+\-])\s*(\d+)\s+days?$").unwrap());
    // ── "what day is <date>" ─────────────────────────────────
    static RE_WHAT_DAY: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^what\s+day\s+(?:is\s+)?(.+)$").unwrap());
    // ── "week number of <date>" / "week number <date>" ───────
    static RE_WEEK_NUM: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^week\s+(?:number\s+)?(?:of\s+)?(.+)$").unwrap());
    // ── named holidays (Christmas, New Year, etc.) ───────────
    fn resolve_named_date(s: &str) -> Option<NaiveDate> {
        let year = Local::now().date_naive().year();
        let s_lower = s.trim().to_lowercase();
        let (m, d) = match s_lower.as_str() {
            "christmas" | "xmas" => (12, 25),
            "new year" | "new year's" | "new years" | "new year's day" => (1, 1),
            "new year's eve" | "nye" => (12, 31),
            "halloween" => (10, 31),
            "valentine" | "valentine's day" | "valentines day" => (2, 14),
            "st patrick" | "st patrick's day" | "saint patricks day" => (3, 17),
            "april fools" | "april fool's" => (4, 1),
            "independence day" | "4th of july" | "july 4th" => (7, 4),
            "thanksgiving" => return nth_weekday_of_month(year, 11, Weekday::Thu, 4),
            "labour day" | "labor day" => return nth_weekday_of_month(year, 9, Weekday::Mon, 1),
            "memorial day" => return last_weekday_of_month(year, 5, Weekday::Mon),
            "mlk day" | "martin luther king day" => {
                return nth_weekday_of_month(year, 1, Weekday::Mon, 3)
            }
            _ => return parse_date(s),
        };
        let candidate = NaiveDate::from_ymd_opt(year, m, d)?;
        let today = Local::now().date_naive();
        // If already passed this year, give next year
        if candidate < today {
            NaiveDate::from_ymd_opt(year + 1, m, d)
        } else {
            Some(candidate)
        }
    }

    // ── resolve_named_date_no_future: like resolve_named_date but never bumps
    // to next year - used for "since/from" queries where the date being in the
    // past is expected and correct.
    fn resolve_named_date_no_future(s: &str) -> Option<NaiveDate> {
        let year = Local::now().date_naive().year();
        let s_lower = s.trim().to_lowercase();
        let (m, d) = match s_lower.as_str() {
            "christmas" | "xmas" => (12, 25),
            "new year" | "new year's" | "new years" | "new year's day" => (1, 1),
            "new year's eve" | "nye" => (12, 31),
            "halloween" => (10, 31),
            "valentine" | "valentine's day" | "valentines day" => (2, 14),
            "st patrick" | "st patrick's day" | "saint patricks day" => (3, 17),
            "april fools" | "april fool's" => (4, 1),
            "independence day" | "4th of july" | "july 4th" => (7, 4),
            "thanksgiving" => return nth_weekday_of_month(year, 11, Weekday::Thu, 4),
            "labour day" | "labor day" => return nth_weekday_of_month(year, 9, Weekday::Mon, 1),
            "memorial day" => return last_weekday_of_month(year, 5, Weekday::Mon),
            "mlk day" | "martin luther king day" => {
                return nth_weekday_of_month(year, 1, Weekday::Mon, 3)
            }
            _ => return parse_date(s),
        };
        // Always return the current-year date, never bump to next year
        NaiveDate::from_ymd_opt(year, m, d)
    }

    if let Some(caps) = RE_UNTIL.captures(q) {
        let target = resolve_named_date(caps[2].trim())?;
        let diff = (target - today).num_days();
        let human = humanize_duration(diff);
        let label = if diff == 0 {
            "today!".to_string()
        } else if diff == 1 {
            "tomorrow".to_string()
        } else {
            format!("{} days ({}) until {}", diff, human, format_date(target))
        };
        results.push(make_date_result(label, diff.to_string()));
        return Some(results);
    }

    if let Some(caps) = RE_SINCE.captures(q) {
        // Use no-future variant so "days since 12 jan" always refers to the
        // past date, not next year's occurrence.
        let from = resolve_named_date_no_future(caps[1].trim())?;
        let diff = (today - from).num_days();
        if diff < 0 {
            // Date is in the future
            let label = format!(
                "{} is in the future ({} days away)",
                format_date(from),
                diff.abs()
            );
            results.push(make_date_result(label, diff.abs().to_string()));
        } else {
            let human = humanize_duration(diff);
            let label = format!("{} days ({}) since {}", diff, human, format_date(from));
            results.push(make_date_result(label, diff.to_string()));
        }
        return Some(results);
    }

    if let Some(caps) = RE_ADD_DAYS.captures(q) {
        let n: i64 = caps[1].parse().ok()?;
        let from = resolve_named_date(caps[2].trim())?;
        let result_date = from + Duration::days(n);
        let label = format!(
            "{} + {} days = {} ({})",
            format_date(from),
            n,
            format_date(result_date),
            weekday_name(result_date.weekday())
        );
        let copy = result_date.format("%Y-%m-%d").to_string();
        results.push(make_date_result(label, copy));
        return Some(results);
    }

    if let Some(caps) = RE_SUB_DAYS.captures(q) {
        let n: i64 = caps[1].parse().ok()?;
        let from = resolve_named_date(caps[2].trim())?;
        let result_date = from - Duration::days(n);
        let label = format!(
            "{} - {} days = {} ({})",
            format_date(from),
            n,
            format_date(result_date),
            weekday_name(result_date.weekday())
        );
        let copy = result_date.format("%Y-%m-%d").to_string();
        results.push(make_date_result(label, copy));
        return Some(results);
    }

    if let Some(caps) = RE_DATE_ARITH.captures(q) {
        let date_str = caps[1].trim();
        let op = &caps[2];
        let n: i64 = caps[3].parse().ok()?;
        let from = resolve_named_date(date_str)?;
        let result_date = if op == "+" {
            from + Duration::days(n)
        } else {
            from - Duration::days(n)
        };
        let label = format!(
            "{} {} {} days = {} ({})",
            format_date(from),
            op,
            n,
            format_date(result_date),
            weekday_name(result_date.weekday())
        );
        let copy = result_date.format("%Y-%m-%d").to_string();
        results.push(make_date_result(label, copy));
        return Some(results);
    }

    if let Some(caps) = RE_WHAT_DAY.captures(q) {
        let target = resolve_named_date(caps[1].trim())?;
        let wd = weekday_name(target.weekday());
        let label = format!("{} is a {}", format_date(target), wd);
        results.push(make_date_result(label, wd.to_string()));
        return Some(results);
    }

    if let Some(caps) = RE_WEEK_NUM.captures(q) {
        let target = resolve_named_date(caps[1].trim())?;
        let wn = target.iso_week().week();
        let label = format!(
            "{} is in week {} of {}",
            format_date(target),
            wn,
            target.year()
        );
        results.push(make_date_result(label, wn.to_string()));
        return Some(results);
    }

    // ── "<date> to/- <date>" (between) - try last so it doesn't swallow other patterns ──
    if let Some(caps) = RE_BETWEEN.captures(q) {
        let a = resolve_named_date(caps[1].trim());
        let b = resolve_named_date(caps[2].trim());
        if let (Some(d1), Some(d2)) = (a, b) {
            let diff = (d2 - d1).num_days().abs();
            let human = humanize_duration(diff);
            let (earlier, later) = if d1 <= d2 { (d1, d2) } else { (d2, d1) };
            let label = format!(
                "{} → {} = {} days ({})",
                format_date(earlier),
                format_date(later),
                diff,
                human
            );
            results.push(make_date_result(label, diff.to_string()));
            return Some(results);
        }
    }

    // ── Bare date - show info about it ───────────────────────
    if let Some(target) = parse_date(q) {
        let diff = (target - today).num_days();
        let wd = weekday_name(target.weekday());
        let wn = target.iso_week().week();
        let human = humanize_duration(diff.abs());
        let direction = if diff > 0 {
            format!("in {}", human)
        } else if diff < 0 {
            format!("{} ago", human)
        } else {
            "today".to_string()
        };
        let label = format!(
            "{} - {} (week {}) - {}",
            format_date(target),
            wd,
            wn,
            direction
        );
        let iso = target.format("%Y-%m-%d").to_string();
        results.push(make_date_result(label, iso));
        return Some(results);
    }

    None
}

/// Return the nth occurrence of a weekday in a given month/year (e.g. 4th Thursday in November).
fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> Option<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let first_wd = first.weekday();
    let delta = (weekday_num(weekday) as i64 - weekday_num(first_wd) as i64).rem_euclid(7);
    let day = 1 + delta + (n as i64 - 1) * 7;
    NaiveDate::from_ymd_opt(year, month, day as u32)
}

/// Return the last occurrence of a weekday in a given month/year.
fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> Option<NaiveDate> {
    // Last day of the month
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)? - Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)? - Duration::days(1)
    };
    let last_wd = last_day.weekday();
    let delta = (weekday_num(last_wd) as i64 - weekday_num(weekday) as i64).rem_euclid(7);
    Some(last_day - Duration::days(delta))
}
