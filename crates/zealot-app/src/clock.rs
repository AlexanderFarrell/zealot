//! The user's local clock.
//!
//! The time zone is the account setting `timezone` (an IANA name such as `Europe/London`), then
//! the `ZEALOT_TIMEZONE` environment variable, then UTC. Rules (Lua) and item queries share this
//! so "today" means the same day everywhere.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use zealot_domain::account::Account;

/// Resolve the time zone from account settings, falling back to `ZEALOT_TIMEZONE`, then UTC.
pub fn timezone_for_settings(settings: &serde_json::Value) -> Tz {
    settings
        .get("timezone")
        .and_then(|v| v.as_str())
        .and_then(parse_timezone)
        .or_else(|| {
            std::env::var("ZEALOT_TIMEZONE")
                .ok()
                .and_then(|v| parse_timezone(&v))
        })
        .unwrap_or(Tz::UTC)
}

/// The account's time zone.
pub fn timezone_for(account: &Account) -> Tz {
    timezone_for_settings(&account.settings)
}

/// The local calendar date at `now` in `tz`.
pub fn local_date_at(now: DateTime<Utc>, tz: Tz) -> NaiveDate {
    now.with_timezone(&tz).date_naive()
}

/// Today in the account's time zone.
pub fn local_today(account: &Account) -> NaiveDate {
    local_date_at(Utc::now(), timezone_for(account))
}

/// Parse an IANA time zone name, ignoring surrounding whitespace.
pub fn parse_timezone(name: &str) -> Option<Tz> {
    name.trim().parse::<Tz>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn account_setting_wins_and_shifts_the_date() {
        let tz = timezone_for_settings(&json!({"timezone": "Europe/London"}));
        assert_eq!(tz, Tz::Europe__London);
        // 23:30 UTC in summer is already the next day in London.
        let now = Utc.with_ymd_and_hms(2026, 7, 1, 23, 30, 0).unwrap();
        assert_eq!(
            local_date_at(now, tz),
            NaiveDate::from_ymd_opt(2026, 7, 2).unwrap()
        );
    }

    #[test]
    fn invalid_setting_falls_through() {
        // An unknown zone name is ignored rather than failing the query.
        let tz = timezone_for_settings(&json!({"timezone": "Mars/Olympus"}));
        assert!(tz == Tz::UTC || std::env::var("ZEALOT_TIMEZONE").is_ok());
    }
}
