//! Relative date tokens in item query filter values.
//!
//! A saved query must keep meaning "today" tomorrow, so filter values may use
//! tokens that resolve against the caller's local date when the query runs:
//!
//! | Token | Resolves to | Example (on 2026-10-02) |
//! | --- | --- | --- |
//! | `$today`, `$today+Nd`, `$today-Nd` | `YYYY-MM-DD` | `$today-30d` → `2026-09-02` |
//! | `$this_week`, `$this_week+Nw`, `$this_week-Nw` | ISO week `YYYY-Wnn` | `$this_week+1w` → `2026-W41` |
//! | `$this_month` | month number (integer, matches `Month`) | `10` |
//! | `$this_year` | year (integer, matches `Year`) | `2026` |
//!
//! Strings that do not start with one of these token names pass through unchanged, so a literal
//! such as `"$500"` is still a plain value. A known token with a malformed offset is an error,
//! so a typo never silently matches nothing.

use chrono::{Datelike, Duration, NaiveDate};
use serde_json::Value;

use crate::{attribute::AttributeFilterDto, item::ItemQueryDto};

const TODAY: &str = "$today";
const THIS_WEEK: &str = "$this_week";
const THIS_MONTH: &str = "$this_month";
const THIS_YEAR: &str = "$this_year";

/// Resolve every token in a filter value (a scalar or an array of scalars) against `today`.
pub fn resolve_value(value: &Value, today: NaiveDate) -> Result<Value, String> {
    match value {
        Value::String(text) => resolve_text(text, today),
        Value::Array(values) => values
            .iter()
            .map(|v| resolve_value(v, today))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        other => Ok(other.clone()),
    }
}

/// A copy of `query` with every filter value's tokens resolved, as it will actually execute.
pub fn resolve_query(query: &ItemQueryDto, today: NaiveDate) -> Result<ItemQueryDto, String> {
    let resolve = |filters: &[AttributeFilterDto]| -> Result<Vec<AttributeFilterDto>, String> {
        filters
            .iter()
            .map(|f| {
                Ok(AttributeFilterDto {
                    value: resolve_value(&f.value, today)?,
                    ..f.clone()
                })
            })
            .collect()
    };
    let mut resolved = query.clone();
    resolved.filters = resolve(&query.filters)?;
    resolved.any_of = query
        .any_of
        .iter()
        .map(|group| resolve(group))
        .collect::<Result<_, _>>()?;
    Ok(resolved)
}

/// True when the value contains at least one relative date token.
pub fn has_token(value: &Value) -> bool {
    match value {
        Value::String(text) => token_name(text).is_some(),
        Value::Array(values) => values.iter().any(has_token),
        _ => false,
    }
}

/// The token name a string starts with, if any.
fn token_name(text: &str) -> Option<&'static str> {
    [THIS_MONTH, THIS_WEEK, THIS_YEAR, TODAY]
        .into_iter()
        .find(|name| text.starts_with(name))
}

fn resolve_text(text: &str, today: NaiveDate) -> Result<Value, String> {
    let Some(name) = token_name(text) else {
        return Ok(Value::String(text.to_string()));
    };
    let suffix = &text[name.len()..];
    match name {
        TODAY => {
            let days = parse_offset(text, suffix, 'd')?;
            let date = today + Duration::days(days);
            Ok(Value::String(date.format("%Y-%m-%d").to_string()))
        }
        THIS_WEEK => {
            let weeks = parse_offset(text, suffix, 'w')?;
            let week = (today + Duration::weeks(weeks)).iso_week();
            Ok(Value::String(format!(
                "{}-W{:02}",
                week.year(),
                week.week()
            )))
        }
        THIS_MONTH => {
            require_no_suffix(text, suffix)?;
            Ok(Value::from(today.month()))
        }
        _ => {
            require_no_suffix(text, suffix)?;
            Ok(Value::from(today.year()))
        }
    }
}

/// Parse an optional `+N<unit>` or `-N<unit>` offset; an empty suffix is zero.
fn parse_offset(token: &str, suffix: &str, unit: char) -> Result<i64, String> {
    if suffix.is_empty() {
        return Ok(0);
    }
    let invalid =
        || format!("invalid date token '{token}': expected an offset like +7{unit} or -7{unit}");
    let (sign, rest) = match suffix.chars().next() {
        Some('+') => (1, &suffix[1..]),
        Some('-') => (-1, &suffix[1..]),
        _ => return Err(invalid()),
    };
    let digits = rest.strip_suffix(unit).ok_or_else(invalid)?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(invalid());
    }
    let n: i64 = digits.parse().map_err(|_| invalid())?;
    // Bound offsets so date arithmetic can never overflow.
    if n > 100_000 {
        return Err(invalid());
    }
    Ok(sign * n)
}

fn require_no_suffix(token: &str, suffix: &str) -> Result<(), String> {
    if suffix.is_empty() {
        Ok(())
    } else {
        Err(format!("invalid date token '{token}': it takes no offset"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 2).unwrap()
    }

    #[test]
    fn resolves_day_tokens() {
        assert_eq!(
            resolve_value(&json!("$today"), day()).unwrap(),
            json!("2026-10-02")
        );
        assert_eq!(
            resolve_value(&json!("$today-30d"), day()).unwrap(),
            json!("2026-09-02")
        );
        assert_eq!(
            resolve_value(&json!("$today+1d"), day()).unwrap(),
            json!("2026-10-03")
        );
    }

    #[test]
    fn resolves_week_month_year_tokens() {
        assert_eq!(
            resolve_value(&json!("$this_week"), day()).unwrap(),
            json!("2026-W40")
        );
        assert_eq!(
            resolve_value(&json!("$this_week+1w"), day()).unwrap(),
            json!("2026-W41")
        );
        assert_eq!(
            resolve_value(&json!("$this_month"), day()).unwrap(),
            json!(10)
        );
        assert_eq!(
            resolve_value(&json!("$this_year"), day()).unwrap(),
            json!(2026)
        );
    }

    #[test]
    fn iso_week_crosses_the_year_boundary() {
        let new_year = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
        assert_eq!(
            resolve_value(&json!("$this_week"), new_year).unwrap(),
            json!("2026-W53")
        );
    }

    #[test]
    fn resolves_inside_arrays_and_leaves_literals() {
        assert_eq!(
            resolve_value(&json!(["$today", "Working", 5, "$500"]), day()).unwrap(),
            json!(["2026-10-02", "Working", 5, "$500"])
        );
        assert!(has_token(&json!(["x", "$this_week"])));
        assert!(!has_token(&json!("$500")));
    }

    #[test]
    fn rejects_malformed_offsets() {
        for bad in [
            "$today+",
            "$today+3",
            "$today*3d",
            "$today+3w",
            "$today+-3d",
            "$this_month+1",
            "$this_week+1d",
            "$today+999999999d",
        ] {
            assert!(
                resolve_value(&json!(bad), day()).is_err(),
                "{bad} should fail"
            );
        }
    }
}
