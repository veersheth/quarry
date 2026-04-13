use once_cell::sync::Lazy;
use regex::Regex;
use tauri::AppHandle;
use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};

pub struct MathSearcher;

impl SearchProvider for MathSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let expr = query.trim();
        let mut results = Vec::new();

        if let Some(items) = try_unit_conversion(expr) {
            results.extend(items);
        } else if let Some(item) = try_percentage(expr) {
            results.push(item);
        } else if let Some(item) = try_math(expr) {
            results.push(item);
        }

        SearchResult { results, result_type: ResultType::Math }
    }
}

// ============================================================
// Number formatting
// ============================================================

fn format_num(n: f64) -> String {
    if n.is_nan() { return "NaN".into(); }
    if n.is_infinite() { return if n > 0.0 { "∞".into() } else { "-∞".into() }; }

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

    let precision = if abs >= 1000.0 { 3 } else if abs >= 1.0 { 6 } else { 9 };
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
    let (sign, digits) = if s.starts_with('-') { ("-", &s[1..]) } else { ("", s) };
    let mut result = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 { result.push(','); }
        result.push(c);
    }
    format!("{}{}", sign, result)
}

// ============================================================
// Result builder
// ============================================================

fn make_result(name: String, copy: String) -> ResultItem {
    ResultItem::new(name, ActionData::CopyToClipboard { text: copy })
        .description("Copy to clipboard")
        .icon("icons/math.png")
}

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
    if expr.is_empty() { return None; }
    let preprocessed = preprocess_math(expr);

    let mut ctx = meval::Context::new();

    ctx.func("fact", |x: f64| -> f64 {
        if x < 0.0 || x.fract() != 0.0 || x > 20.0 { return f64::NAN; }
        (1..=(x as u64)).map(|i| i as f64).product()
    });
    ctx.func("cbrt", |x: f64| x.cbrt());
    ctx.func("deg", |x: f64| x.to_degrees());
    ctx.func("rad", |x: f64| x.to_radians());
    // log(base, x)
    ctx.func2("log", |base: f64, x: f64| x.log(base));
    // nPr(n, r) — permutations
    ctx.func2("nPr", |n: f64, r: f64| -> f64 {
        if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 { return f64::NAN; }
        let (n, r) = (n as u64, r as u64);
        ((n - r + 1)..=n).map(|i| i as f64).product()
    });
    // nCr(n, r) — combinations
    ctx.func2("nCr", |n: f64, r: f64| -> f64 {
        if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 { return f64::NAN; }
        let (n, r) = (n as u64, r as u64);
        let r = r.min(n - r);
        let mut res = 1.0f64;
        for i in 0..r { res = res * (n - i) as f64 / (i + 1) as f64; }
        res
    });
    ctx.func2("gcd", |a: f64, b: f64| -> f64 {
        let (mut a, mut b) = (a.abs() as u64, b.abs() as u64);
        while b != 0 { let t = b; b = a % b; a = t; }
        a as f64
    });
    ctx.func2("lcm", |a: f64, b: f64| -> f64 {
        let (au, bu) = (a.abs() as u64, b.abs() as u64);
        if au == 0 || bu == 0 { return 0.0; }
        let (mut g, mut gb) = (au, bu);
        while gb != 0 { let t = gb; gb = g % gb; g = t; }
        (au / g * bu) as f64
    });
    ctx.var("tau", std::f64::consts::TAU);
    ctx.var("phi", 1.618_033_988_749_895_f64);

    match meval::eval_str_with_context(&preprocessed, ctx) {
        Ok(val) => {
            let formatted = format_num(val);
            let name = format!("{} = {}", expr, formatted);
            Some(make_result(name, val.to_string()))
        }
        Err(_) => None,
    }
}

// ============================================================
// Percentage calculations
// ============================================================

fn try_percentage(expr: &str) -> Option<ResultItem> {
    static RE_OF:   Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^([\d.]+)%\s*of\s*([\d.,]+)$").unwrap());
    static RE_OFF:  Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^([\d.]+)%\s*off\s*([\d.,]+)$").unwrap());
    static RE_WHAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^([\d.,]+)\s+is\s+what\s*%\s*of\s*([\d.,]+)$").unwrap());
    static RE_ADD:  Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^([\d.,]+)\s*\+\s*([\d.]+)%$").unwrap());

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
        let name = format!("{}% off {} = {} (save {})", pct, format_num(total), format_num(val), format_num(discount));
        return Some(make_result(name, format_num(val)));
    }

    if let Some(caps) = RE_WHAT.captures(expr) {
        let part: f64 = caps[1].replace(',', "").parse().ok()?;
        let total: f64 = caps[2].replace(',', "").parse().ok()?;
        if total == 0.0 { return None; }
        let pct = part / total * 100.0;
        let name = format!("{} is {}% of {}", format_num(part), format_num(pct), format_num(total));
        return Some(make_result(name, format!("{}%", format_num(pct))));
    }

    if let Some(caps) = RE_ADD.captures(expr) {
        let base: f64 = caps[1].replace(',', "").parse().ok()?;
        let pct: f64 = caps[2].parse().ok()?;
        let val = base * (1.0 + pct / 100.0);
        let added = base * pct / 100.0;
        let name = format!("{} + {}% = {} (+ {})", format_num(base), pct, format_num(val), format_num(added));
        return Some(make_result(name, format_num(val)));
    }

    None
}

// ============================================================
// Unit conversion
// ============================================================

struct UnitDef {
    aliases:  &'static [&'static str],
    display:  &'static str,
    category: &'static str,
    to_base:  f64,
}

static UNITS: Lazy<Vec<UnitDef>> = Lazy::new(|| vec![
    // ── Length (base: meters) ──────────────────────────────
    UnitDef { aliases: &["mm", "millimeter", "millimeters", "millimetre", "millimetres"], display: "mm",   category: "length", to_base: 0.001 },
    UnitDef { aliases: &["cm", "centimeter", "centimeters", "centimetre", "centimetres"], display: "cm",   category: "length", to_base: 0.01 },
    UnitDef { aliases: &["m", "meter", "meters", "metre", "metres"],                     display: "m",    category: "length", to_base: 1.0 },
    UnitDef { aliases: &["km", "kilometer", "kilometers", "kilometre", "kilometres"],     display: "km",   category: "length", to_base: 1_000.0 },
    UnitDef { aliases: &["in", "inch", "inches"],                                         display: "in",   category: "length", to_base: 0.0254 },
    UnitDef { aliases: &["ft", "foot", "feet"],                                           display: "ft",   category: "length", to_base: 0.3048 },
    UnitDef { aliases: &["yd", "yard", "yards"],                                          display: "yd",   category: "length", to_base: 0.9144 },
    UnitDef { aliases: &["mi", "mile", "miles"],                                          display: "mi",   category: "length", to_base: 1_609.344 },
    UnitDef { aliases: &["nmi", "nautical mile", "nautical miles"],                       display: "nmi",  category: "length", to_base: 1_852.0 },
    UnitDef { aliases: &["ly", "light year", "light years", "lightyear", "lightyears"],  display: "ly",   category: "length", to_base: 9.461e15 },
    UnitDef { aliases: &["au", "astronomical unit", "astronomical units"],                display: "AU",   category: "length", to_base: 1.496e11 },

    // ── Weight (base: grams) ──────────────────────────────
    UnitDef { aliases: &["mg", "milligram", "milligrams"],                               display: "mg",  category: "weight", to_base: 0.001 },
    UnitDef { aliases: &["g", "gram", "grams", "gramme", "grammes"],                     display: "g",   category: "weight", to_base: 1.0 },
    UnitDef { aliases: &["kg", "kilogram", "kilograms", "kilogramme", "kilogrammes"],    display: "kg",  category: "weight", to_base: 1_000.0 },
    UnitDef { aliases: &["t", "tonne", "tonnes", "metric ton", "metric tons"],           display: "t",   category: "weight", to_base: 1_000_000.0 },
    UnitDef { aliases: &["oz", "ounce", "ounces"],                                       display: "oz",  category: "weight", to_base: 28.349_523 },
    UnitDef { aliases: &["lb", "lbs", "pound", "pounds"],                                display: "lb",  category: "weight", to_base: 453.592_37 },
    UnitDef { aliases: &["st", "stone", "stones"],                                       display: "st",  category: "weight", to_base: 6_350.293_18 },

    // ── Temperature — special (aliases only, conversion handled separately) ──
    UnitDef { aliases: &["c", "°c", "celsius", "centigrade"],  display: "°C", category: "temperature", to_base: 1.0 },
    UnitDef { aliases: &["f", "°f", "fahrenheit"],             display: "°F", category: "temperature", to_base: 1.0 },
    UnitDef { aliases: &["k", "kelvin"],                       display: "K",  category: "temperature", to_base: 1.0 },

    // ── Volume (base: millilitres) ────────────────────────
    UnitDef { aliases: &["ml", "milliliter", "milliliters", "millilitre", "millilitres"], display: "ml",   category: "volume", to_base: 1.0 },
    UnitDef { aliases: &["cl", "centiliter", "centiliters", "centilitre", "centilitres"], display: "cl",   category: "volume", to_base: 10.0 },
    UnitDef { aliases: &["l", "liter", "liters", "litre", "litres"],                     display: "L",    category: "volume", to_base: 1_000.0 },
    UnitDef { aliases: &["tsp", "teaspoon", "teaspoons"],                                 display: "tsp",  category: "volume", to_base: 4.928_92 },
    UnitDef { aliases: &["tbsp", "tablespoon", "tablespoons"],                            display: "tbsp", category: "volume", to_base: 14.786_8 },
    UnitDef { aliases: &["fl oz", "floz", "fluid oz", "fluid ounce", "fluid ounces"],    display: "fl oz",category: "volume", to_base: 29.573_5 },
    UnitDef { aliases: &["cup", "cups"],                                                  display: "cup",  category: "volume", to_base: 236.588 },
    UnitDef { aliases: &["pt", "pint", "pints"],                                          display: "pt",   category: "volume", to_base: 473.176 },
    UnitDef { aliases: &["qt", "quart", "quarts"],                                        display: "qt",   category: "volume", to_base: 946.353 },
    UnitDef { aliases: &["gal", "gallon", "gallons"],                                     display: "gal",  category: "volume", to_base: 3_785.41 },

    // ── Area (base: m²) ───────────────────────────────────
    UnitDef { aliases: &["mm2", "mm²", "sq mm", "square millimeter", "square millimeters"], display: "mm²",  category: "area", to_base: 0.000_001 },
    UnitDef { aliases: &["cm2", "cm²", "sq cm", "square centimeter", "square centimeters"], display: "cm²",  category: "area", to_base: 0.000_1 },
    UnitDef { aliases: &["m2", "m²", "sq m", "square meter", "square meters"],              display: "m²",   category: "area", to_base: 1.0 },
    UnitDef { aliases: &["km2", "km²", "sq km", "square kilometer", "square kilometers"],   display: "km²",  category: "area", to_base: 1_000_000.0 },
    UnitDef { aliases: &["in2", "in²", "sq in", "square inch", "square inches"],            display: "in²",  category: "area", to_base: 0.000_645_16 },
    UnitDef { aliases: &["ft2", "ft²", "sq ft", "square foot", "square feet"],              display: "ft²",  category: "area", to_base: 0.092_903_04 },
    UnitDef { aliases: &["yd2", "yd²", "sq yd", "square yard", "square yards"],             display: "yd²",  category: "area", to_base: 0.836_127_36 },
    UnitDef { aliases: &["acre", "acres"],                                                   display: "acre", category: "area", to_base: 4_046.856_4 },
    UnitDef { aliases: &["ha", "hectare", "hectares"],                                       display: "ha",   category: "area", to_base: 10_000.0 },

    // ── Speed (base: m/s) ─────────────────────────────────
    UnitDef { aliases: &["m/s", "mps", "meters per second", "metres per second"],  display: "m/s",  category: "speed", to_base: 1.0 },
    UnitDef { aliases: &["km/h", "kmh", "kph", "kilometers per hour"],             display: "km/h", category: "speed", to_base: 1.0 / 3.6 },
    UnitDef { aliases: &["mph", "miles per hour", "mi/h"],                          display: "mph",  category: "speed", to_base: 0.447_04 },
    UnitDef { aliases: &["knot", "knots", "kt", "kn"],                             display: "knot", category: "speed", to_base: 0.514_444 },
    UnitDef { aliases: &["ft/s", "fps", "feet per second"],                         display: "ft/s", category: "speed", to_base: 0.3048 },

    // ── Time (base: seconds) ──────────────────────────────
    UnitDef { aliases: &["ns", "nanosecond", "nanoseconds"],    display: "ns",    category: "time", to_base: 1e-9 },
    UnitDef { aliases: &["us", "µs", "microsecond", "microseconds"], display: "µs", category: "time", to_base: 1e-6 },
    UnitDef { aliases: &["ms", "millisecond", "milliseconds"],  display: "ms",    category: "time", to_base: 0.001 },
    UnitDef { aliases: &["s", "sec", "second", "seconds"],      display: "s",     category: "time", to_base: 1.0 },
    UnitDef { aliases: &["min", "minute", "minutes"],            display: "min",   category: "time", to_base: 60.0 },
    UnitDef { aliases: &["h", "hr", "hour", "hours"],           display: "hr",    category: "time", to_base: 3_600.0 },
    UnitDef { aliases: &["d", "day", "days"],                    display: "day",   category: "time", to_base: 86_400.0 },
    UnitDef { aliases: &["wk", "week", "weeks"],                 display: "week",  category: "time", to_base: 604_800.0 },
    UnitDef { aliases: &["mo", "month", "months"],               display: "month", category: "time", to_base: 2_629_800.0 },
    UnitDef { aliases: &["yr", "year", "years"],                 display: "year",  category: "time", to_base: 31_557_600.0 },

    // ── Data (base: bytes) ────────────────────────────────
    UnitDef { aliases: &["bit", "bits"],         display: "bit",  category: "data", to_base: 0.125 },
    UnitDef { aliases: &["byte", "bytes"],        display: "byte", category: "data", to_base: 1.0 },
    UnitDef { aliases: &["kb", "kilobyte", "kilobytes"],     display: "KB",  category: "data", to_base: 1_000.0 },
    UnitDef { aliases: &["kib", "kibibyte", "kibibytes"],    display: "KiB", category: "data", to_base: 1_024.0 },
    UnitDef { aliases: &["mb", "megabyte", "megabytes"],     display: "MB",  category: "data", to_base: 1_000_000.0 },
    UnitDef { aliases: &["mib", "mebibyte", "mebibytes"],   display: "MiB", category: "data", to_base: 1_048_576.0 },
    UnitDef { aliases: &["gb", "gigabyte", "gigabytes"],     display: "GB",  category: "data", to_base: 1_000_000_000.0 },
    UnitDef { aliases: &["gib", "gibibyte", "gibibytes"],    display: "GiB", category: "data", to_base: 1_073_741_824.0 },
    UnitDef { aliases: &["tb", "terabyte", "terabytes"],     display: "TB",  category: "data", to_base: 1_000_000_000_000.0 },
    UnitDef { aliases: &["tib", "tebibyte", "tebibytes"],    display: "TiB", category: "data", to_base: 1_099_511_627_776.0 },
    UnitDef { aliases: &["pb", "petabyte", "petabytes"],     display: "PB",  category: "data", to_base: 1e15 },

    // ── Pressure (base: Pa) ───────────────────────────────
    UnitDef { aliases: &["pa", "pascal", "pascals"],         display: "Pa",   category: "pressure", to_base: 1.0 },
    UnitDef { aliases: &["kpa", "kilopascal", "kilopascals"],display: "kPa",  category: "pressure", to_base: 1_000.0 },
    UnitDef { aliases: &["mpa", "megapascal", "megapascals"],display: "MPa",  category: "pressure", to_base: 1_000_000.0 },
    UnitDef { aliases: &["bar"],                              display: "bar",  category: "pressure", to_base: 100_000.0 },
    UnitDef { aliases: &["mbar", "millibar", "millibars"],   display: "mbar", category: "pressure", to_base: 100.0 },
    UnitDef { aliases: &["psi"],                              display: "psi",  category: "pressure", to_base: 6_894.757 },
    UnitDef { aliases: &["atm", "atmosphere", "atmospheres"],display: "atm",  category: "pressure", to_base: 101_325.0 },
    UnitDef { aliases: &["mmhg", "torr"],                    display: "mmHg", category: "pressure", to_base: 133.322 },

    // ── Energy (base: joules) ─────────────────────────────
    UnitDef { aliases: &["j", "joule", "joules"],              display: "J",    category: "energy", to_base: 1.0 },
    UnitDef { aliases: &["kj", "kilojoule", "kilojoules"],     display: "kJ",   category: "energy", to_base: 1_000.0 },
    UnitDef { aliases: &["mj", "megajoule", "megajoules"],     display: "MJ",   category: "energy", to_base: 1_000_000.0 },
    UnitDef { aliases: &["cal", "calorie", "calories"],         display: "cal",  category: "energy", to_base: 4.184 },
    UnitDef { aliases: &["kcal", "kilocalorie", "kilocalories"],display: "kcal", category: "energy", to_base: 4_184.0 },
    UnitDef { aliases: &["wh", "watt-hour", "watt hour", "watt hours"],    display: "Wh",  category: "energy", to_base: 3_600.0 },
    UnitDef { aliases: &["kwh", "kilowatt-hour", "kilowatt hour", "kwh"],  display: "kWh", category: "energy", to_base: 3_600_000.0 },
    UnitDef { aliases: &["btu"],                               display: "BTU",  category: "energy", to_base: 1_055.06 },
    UnitDef { aliases: &["ev", "electronvolt", "electronvolts"],display: "eV",  category: "energy", to_base: 1.602_176_634e-19 },

    // ── Power (base: watts) ───────────────────────────────
    UnitDef { aliases: &["w", "watt", "watts"],       display: "W",  category: "power", to_base: 1.0 },
    UnitDef { aliases: &["kw", "kilowatt", "kilowatts"], display: "kW", category: "power", to_base: 1_000.0 },
    UnitDef { aliases: &["mw", "megawatt", "megawatts"], display: "MW", category: "power", to_base: 1_000_000.0 },
    UnitDef { aliases: &["hp", "horsepower"],           display: "hp", category: "power", to_base: 745.699_87 },

    // ── Angle (base: radians) ─────────────────────────────
    UnitDef { aliases: &["rad", "radian", "radians"],             display: "rad",  category: "angle", to_base: 1.0 },
    UnitDef { aliases: &["deg", "degree", "degrees", "°"],        display: "°",    category: "angle", to_base: std::f64::consts::PI / 180.0 },
    UnitDef { aliases: &["grad", "gradian", "gradians"],          display: "grad", category: "angle", to_base: std::f64::consts::PI / 200.0 },
    UnitDef { aliases: &["rev", "revolution", "revolutions", "turn", "turns"], display: "rev", category: "angle", to_base: std::f64::consts::TAU },
]);

fn find_unit(s: &str) -> Option<&'static UnitDef> {
    let lower = s.trim().to_lowercase();
    UNITS.iter().find(|u| u.aliases.iter().any(|a| *a == lower.as_str()))
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Option<f64> {
    let is_c = |s: &str| matches!(s, "c" | "°c" | "celsius" | "centigrade");
    let is_f = |s: &str| matches!(s, "f" | "°f" | "fahrenheit");
    let is_k = |s: &str| matches!(s, "k" | "kelvin");

    let celsius = if      is_c(from) { value }
                  else if is_f(from) { (value - 32.0) * 5.0 / 9.0 }
                  else if is_k(from) { value - 273.15 }
                  else               { return None; };

    let result = if      is_c(to) { celsius }
                 else if is_f(to) { celsius * 9.0 / 5.0 + 32.0 }
                 else if is_k(to) { celsius + 273.15 }
                 else             { return None; };

    Some(result)
}

fn parse_num_and_unit(s: &str) -> Option<(f64, &str)> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^([+-]?[\d,]+(?:\.[\d,]+)?(?:[eE][+-]?\d+)?)\s*(.+)$").unwrap()
    });
    let caps = RE.captures(s.trim())?;
    let num_str = caps.get(1)?.as_str().replace(',', "");
    let value: f64 = num_str.parse().ok()?;
    let unit_start = caps.get(2)?.start();
    Some((value, s.trim()[unit_start..].trim()))
}

fn try_unit_conversion(query: &str) -> Option<Vec<ResultItem>> {
    let q = query.trim();

    // Split on " to ", " in ", " -> ", " → "
    let separators = [" to ", " in ", " -> ", " → "];
    let (from_part, to_part) = separators.iter().find_map(|sep| {
        let lower = q.to_lowercase();
        lower.find(sep).map(|pos| (&q[..pos], q[pos + sep.len()..].trim()))
    })?;

    let (value, from_str) = parse_num_and_unit(from_part)?;
    let to_str = to_part.trim();

    // Temperature (non-linear)
    let from_lower = from_str.to_lowercase();
    let to_lower   = to_str.to_lowercase();
    let is_temp    = |s: &str| matches!(s, "c"|"°c"|"celsius"|"centigrade"|"f"|"°f"|"fahrenheit"|"k"|"kelvin");

    if is_temp(&from_lower) && is_temp(&to_lower) {
        let result_val = convert_temperature(value, &from_lower, &to_lower)?;
        let from_u = find_unit(&from_lower).map(|u| u.display).unwrap_or(from_str);
        let to_u   = find_unit(&to_lower).map(|u| u.display).unwrap_or(to_str);
        let formatted = format_num(result_val);
        let name = format!("{} {} = {} {}", format_num(value), from_u, formatted, to_u);
        return Some(vec![make_result(name, format!("{} {}", formatted, to_u))]);
    }

    let from_unit = find_unit(&from_lower)?;
    let to_unit   = find_unit(&to_lower)?;

    if from_unit.category != to_unit.category { return None; }

    let base_value = value * from_unit.to_base;
    let result_val = base_value / to_unit.to_base;
    let formatted  = format_num(result_val);
    let name = format!("{} {} = {} {}", format_num(value), from_unit.display, formatted, to_unit.display);

    Some(vec![make_result(name, format!("{} {}", formatted, to_unit.display))])
}
