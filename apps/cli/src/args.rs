//! Shared argument types: fuzzy dates, item references, time ranges.

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};

/// An item referenced by numeric id or by title.
#[derive(Debug, Clone)]
pub enum ItemRef {
    Id(i64),
    Title(String),
}

impl std::str::FromStr for ItemRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("item reference cannot be empty".into());
        }
        match s.parse::<i64>() {
            Ok(id) if id > 0 => Ok(ItemRef::Id(id)),
            _ => Ok(ItemRef::Title(s.to_string())),
        }
    }
}

impl std::fmt::Display for ItemRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemRef::Id(id) => write!(f, "#{id}"),
            ItemRef::Title(t) => write!(f, "'{t}'"),
        }
    }
}

/// A date parsed from friendly terminal input:
/// `today` (default), `yesterday`, `tomorrow`, `+N`/`-N` (days from today),
/// weekday names (`mon`, `friday` — the next occurrence, including today),
/// or `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy)]
pub struct DateArg(pub NaiveDate);

impl Default for DateArg {
    fn default() -> Self {
        DateArg(Local::now().date_naive())
    }
}

impl std::str::FromStr for DateArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let today = Local::now().date_naive();
        let lower = s.trim().to_lowercase();
        let date = match lower.as_str() {
            "" | "today" | "now" => today,
            "yesterday" | "yest" => today - Duration::days(1),
            "tomorrow" | "tom" | "tmrw" => today + Duration::days(1),
            _ => {
                if let Some(rest) = lower.strip_prefix('+') {
                    let days: i64 = rest.parse().map_err(|_| bad_date(s))?;
                    today + Duration::days(days)
                } else if lower.starts_with('-') && lower[1..].chars().all(|c| c.is_ascii_digit()) {
                    let days: i64 = lower[1..].parse().map_err(|_| bad_date(s))?;
                    today - Duration::days(days)
                } else if let Some(weekday) = parse_weekday(&lower) {
                    next_weekday(today, weekday)
                } else {
                    NaiveDate::parse_from_str(&lower, "%Y-%m-%d").map_err(|_| bad_date(s))?
                }
            }
        };
        Ok(DateArg(date))
    }
}

fn bad_date(s: &str) -> String {
    format!("invalid date '{s}' (try today, tomorrow, fri, +3, or YYYY-MM-DD)")
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tues" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thur" | "thurs" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}

/// The next occurrence of `weekday`, counting today as a candidate.
fn next_weekday(from: NaiveDate, weekday: Weekday) -> NaiveDate {
    let diff = (weekday.num_days_from_monday() as i64
        - from.weekday().num_days_from_monday() as i64)
        .rem_euclid(7);
    from + Duration::days(diff)
}

/// ISO week of a date, formatted as the planner expects (`2026-W27`).
pub fn iso_week_string(date: NaiveDate) -> String {
    let week = date.iso_week();
    format!("{:04}-W{:02}", week.year(), week.week())
}

/// A time-of-day range like `9:00-10:30`, `09:00–10:30`, or `9-10`.
/// Produces minutes from midnight.
#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start_min: i32,
    pub end_min: i32,
}

impl std::str::FromStr for TimeRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || format!("invalid time range '{s}' (try 9:00-10:30)");
        let (start, end) = s
            .split_once('-')
            .or_else(|| s.split_once('–'))
            .ok_or_else(err)?;
        let start_min = parse_clock(start).ok_or_else(err)?;
        let end_min = parse_clock(end).ok_or_else(err)?;
        if end_min <= start_min {
            return Err(format!("end time must be after start time in '{s}'"));
        }
        Ok(TimeRange { start_min, end_min })
    }
}

fn parse_clock(s: &str) -> Option<i32> {
    let s = s.trim();
    let (h, m) = match s.split_once(':') {
        Some((h, m)) => (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?),
        None => (s.parse::<i32>().ok()?, 0),
    };
    if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
        return None;
    }
    Some(h * 60 + m)
}

/// Format minutes-from-midnight as `H:MM`.
pub fn format_clock(min: i32) -> String {
    format!("{}:{:02}", min / 60, min % 60)
}

/// Parse `Key=Value` pairs for `--attr` and `attr set`, coercing values:
/// `true`/`false` → bool, integers/floats → numbers, `[a,b]` → list, else text.
pub fn parse_attr_pair(s: &str) -> Result<(String, serde_json::Value), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("expected KEY=VALUE, got '{s}'"))?;
    if key.trim().is_empty() {
        return Err(format!("empty attribute key in '{s}'"));
    }
    Ok((key.trim().to_string(), coerce_value(value.trim())))
}

pub fn coerce_value(raw: &str) -> serde_json::Value {
    match raw {
        "true" => return serde_json::Value::Bool(true),
        "false" => return serde_json::Value::Bool(false),
        _ => {}
    }
    if let Ok(i) = raw.parse::<i64>() {
        return serde_json::Value::from(i);
    }
    if let Ok(f) = raw.parse::<f64>() {
        return serde_json::Value::from(f);
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let inner = &raw[1..raw.len() - 1];
        let items: Vec<serde_json::Value> = inner
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(coerce_value)
            .collect();
        return serde_json::Value::Array(items);
    }
    serde_json::Value::String(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn item_ref_distinguishes_ids_from_titles() {
        assert!(matches!(ItemRef::from_str("42"), Ok(ItemRef::Id(42))));
        assert!(matches!(ItemRef::from_str("My Note"), Ok(ItemRef::Title(_))));
        // Negative numbers are not valid ids — treat as title.
        assert!(matches!(ItemRef::from_str("-1"), Ok(ItemRef::Title(_))));
    }

    #[test]
    fn date_arg_parses_relative_forms() {
        let today = Local::now().date_naive();
        assert_eq!(DateArg::from_str("today").unwrap().0, today);
        assert_eq!(
            DateArg::from_str("tomorrow").unwrap().0,
            today + Duration::days(1)
        );
        assert_eq!(DateArg::from_str("+3").unwrap().0, today + Duration::days(3));
        assert_eq!(
            DateArg::from_str("2026-07-04").unwrap().0,
            NaiveDate::from_ymd_opt(2026, 7, 4).unwrap()
        );
    }

    #[test]
    fn weekday_resolves_to_upcoming_occurrence() {
        let date = DateArg::from_str("fri").unwrap().0;
        assert_eq!(date.weekday(), Weekday::Fri);
        let today = Local::now().date_naive();
        assert!(date >= today && date < today + Duration::days(7));
    }

    #[test]
    fn time_range_parses_minutes() {
        let r = TimeRange::from_str("9:00-10:30").unwrap();
        assert_eq!((r.start_min, r.end_min), (540, 630));
        let r = TimeRange::from_str("9-10").unwrap();
        assert_eq!((r.start_min, r.end_min), (540, 600));
        assert!(TimeRange::from_str("10:00-9:00").is_err());
    }

    #[test]
    fn attr_pairs_coerce_types() {
        assert_eq!(
            parse_attr_pair("Done=true").unwrap().1,
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            parse_attr_pair("Priority=3").unwrap().1,
            serde_json::json!(3)
        );
        assert_eq!(
            parse_attr_pair("Tags=[a, b]").unwrap().1,
            serde_json::json!(["a", "b"])
        );
        assert_eq!(
            parse_attr_pair("Status=In Progress").unwrap().1,
            serde_json::json!("In Progress")
        );
    }
}
