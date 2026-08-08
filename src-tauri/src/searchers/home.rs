use std::sync::atomic::Ordering;
use tauri::AppHandle;

use crate::types::{Action, ActionData, ResultItem};
use crate::searchers::{time::TimeSearcher, timer, SearchProvider};

/// Build the home-screen widget list shown when the query is empty.
/// Each item is tagged `group: "widget"` so the frontend can render them distinctly.
pub fn home_items(app: &AppHandle) -> Vec<ResultItem> {
    let mut items: Vec<ResultItem> = vec![];

    // ---- Pinned world clocks ----
    let clock_items: Vec<ResultItem> = TimeSearcher
        .search("", app)
        .results
        .into_iter()
        .filter(|r| r.pinned)
        .map(|r| r.group("widget"))
        .collect();
    items.extend(clock_items);

    // ---- Running timers ----
    if let Ok(timers) = timer::ACTIVE_TIMERS.lock() {
        for t in timers.iter() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let rem = t.expires_at.saturating_sub(now);
            let ring_at = format_ring_time(t.expires_at);
            let label = if t.label.is_empty() { "Timer".to_string() } else { t.label.clone() };
            let icon = timer_progress_icon(rem, t.duration_secs);

            items.push(
                ResultItem::new(
                    format!("{} - {} left", label, timer::format_duration(rem)),
                    vec![Action::new("Cancel", ActionData::RunFunction {
                        function_name: "cancel_timer".into(),
                        params: vec![t.id.to_string()],
                    })],
                )
                .description(format!("Rings at {}", ring_at))
                .icon(icon)
                .group("widget"),
            );
        }
    }

    // ---- Active noise ----
    if crate::executor::PINK_NOISE_RUNNING.load(Ordering::Relaxed) {
        items.push(
            ResultItem::new(
                "Pink Noise",
                vec![Action::new("Stop", ActionData::RunFunction {
                    function_name: "toggle_pink_noise".into(),
                    params: vec![],
                })],
            )
            .description("Playing")
            .icon("icons/pink-noise.png")
            .group("widget"),
        );
    }

    if crate::executor::RAIN_NOISE_RUNNING.load(Ordering::Relaxed) {
        items.push(
            ResultItem::new(
                "Rain Noise",
                vec![Action::new("Stop", ActionData::RunFunction {
                    function_name: "toggle_rain_noise".into(),
                    params: vec![],
                })],
            )
            .description("Playing - press Enter to stop")
            .group("widget"),
        );
    }

    items
}

fn format_ring_time(unix_secs: u64) -> String {
    use chrono::{Local, TimeZone};
    Local
        .timestamp_opt(unix_secs as i64, 0)
        .single()
        .map(|dt| dt.format("%-I:%M %p").to_string())
        .unwrap_or_else(|| "??:??".to_string())
}

/// Generate a circular progress-ring SVG data URI.
/// `remaining` and `total` are in seconds; the ring drains clockwise as time passes.
fn timer_progress_icon(remaining: u64, total: u64) -> String {
    let fraction = if total == 0 {
        0.0f64
    } else {
        (remaining as f64 / total as f64).clamp(0.0, 1.0)
    };

    let r = 9.0f64;
    let circ = 2.0 * std::f64::consts::PI * r;
    let filled = fraction * circ;
    let gap    = circ - filled;

    let color = if fraction > 0.5 {
        "rgba(74,222,128,0.9)"
    } else if fraction > 0.2 {
        "rgba(251,191,36,0.9)"
    } else {
        "rgba(248,113,113,0.9)"
    };

    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <circle cx="12" cy="12" r="{r}" fill="none" stroke="rgba(255,255,255,0.12)" stroke-width="2.5"/>
  <circle cx="12" cy="12" r="{r}" fill="none" stroke="{color}" stroke-width="2.5"
          stroke-linecap="round"
          stroke-dasharray="{filled:.2} {gap:.2}"
          transform="rotate(-90 12 12)"/>
</svg>"#,
        r = r, color = color, filled = filled, gap = gap,
    );

    let encoded = svg
        .replace('#', "%23")
        .replace('"', "'");
    format!("data:image/svg+xml,{}", encoded)
}
