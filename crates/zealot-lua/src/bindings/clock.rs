//! Time zone aware date helpers for rules.
//!
//! The clock is the user's configured time zone: the account setting
//! `timezone` (an IANA name such as `Europe/London`), then the
//! `ZEALOT_TIMEZONE` environment variable, then UTC. Lua exposes the
//! result as `zealot.timezone`, `zealot.date`, `zealot.local_now`,
//! `zealot.local_date(timestamp)`, `zealot.date_add(date, days)` and
//! `zealot.week_start(date?)`.

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use mlua::{Lua, Table};
use zealot_app::services::ZealotServices;
use zealot_domain::common::id::Id;

const DATE_FORMAT: &str = "%Y-%m-%d";

/// Resolve the account's time zone, falling back to `ZEALOT_TIMEZONE`, then UTC.
/// Shares the rule with item queries (`zealot_app::clock`).
pub fn resolve_timezone(services: &ZealotServices, account_id: Id) -> Tz {
    let settings = services
        .repos
        .account
        .get_account_by_id(&account_id)
        .ok()
        .flatten()
        .map(|account| account.settings)
        .unwrap_or(serde_json::Value::Null);
    zealot_app::clock::timezone_for_settings(&settings)
}

/// The local calendar date at `now` in `tz`.
pub fn local_date_at(now: DateTime<Utc>, tz: Tz) -> NaiveDate {
    now.with_timezone(&tz).date_naive()
}

/// Today in the user's time zone.
pub fn local_today(tz: Tz) -> NaiveDate {
    local_date_at(Utc::now(), tz)
}

/// Parse an RFC 3339 instant or a stored `YYYY-MM-DD HH:MM:SS` UTC timestamp.
fn parse_instant(text: &str) -> Option<DateTime<Utc>> {
    let text = text.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.with_timezone(&Utc));
    }
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(text, format) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }
    None
}

pub fn parse_date(text: &str) -> mlua::Result<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), DATE_FORMAT).map_err(|_| {
        mlua::Error::RuntimeError(format!("invalid date '{text}' (expected YYYY-MM-DD)"))
    })
}

/// Local date for a stored timestamp, e.g. a comment or `Completed At` value.
pub fn to_local_date(timestamp: &str, tz: Tz) -> Option<String> {
    parse_instant(timestamp).map(|dt| local_date_at(dt, tz).format(DATE_FORMAT).to_string())
}

pub fn date_add(date: &str, days: i64) -> mlua::Result<String> {
    let date = parse_date(date)?;
    let shifted = date
        .checked_add_signed(Duration::days(days))
        .ok_or_else(|| mlua::Error::RuntimeError(String::from("date out of range")))?;
    Ok(shifted.format(DATE_FORMAT).to_string())
}

/// Monday of the ISO week containing `date`.
pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

pub fn register(zealot: &Table, lua: &Lua, tz: Tz) -> mlua::Result<()> {
    zealot.set("timezone", tz.name())?;
    zealot.set("date", local_today(tz).format(DATE_FORMAT).to_string())?;
    zealot.set("local_now", Utc::now().with_timezone(&tz).to_rfc3339())?;

    zealot.set(
        "local_date",
        lua.create_function(move |_, timestamp: String| Ok(to_local_date(&timestamp, tz)))?,
    )?;
    zealot.set(
        "date_add",
        lua.create_function(|_, (date, days): (String, i64)| date_add(&date, days))?,
    )?;
    zealot.set(
        "week_start",
        lua.create_function(move |_, date: Option<String>| {
            let date = match date {
                Some(text) => parse_date(&text)?,
                None => local_today(tz),
            };
            Ok(week_start(date).format(DATE_FORMAT).to_string())
        })?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zealot_app::clock::parse_timezone;

    fn utc(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn london_summer_midnight_hour_is_next_day() {
        // 23:30 UTC on 2026-07-01 is 00:30 BST on 2026-07-02.
        let tz: Tz = "Europe/London".parse().unwrap();
        let date = local_date_at(utc("2026-07-01T23:30:00Z"), tz);
        assert_eq!(date.to_string(), "2026-07-02");
        assert_eq!(
            local_date_at(utc("2026-07-01T23:30:00Z"), Tz::UTC).to_string(),
            "2026-07-01"
        );
    }

    #[test]
    fn local_date_of_stored_timestamp() {
        let tz: Tz = "Europe/London".parse().unwrap();
        assert_eq!(
            to_local_date("2026-07-01 23:30:00", tz).unwrap(),
            "2026-07-02"
        );
        assert_eq!(
            to_local_date("2026-01-01T23:30:00+00:00", tz).unwrap(),
            "2026-01-01"
        );
        assert!(to_local_date("garbage", tz).is_none());
    }

    #[test]
    fn date_math() {
        assert_eq!(date_add("2026-10-01", 7).unwrap(), "2026-10-08");
        assert_eq!(date_add("2026-03-01", -1).unwrap(), "2026-02-28");
        assert!(date_add("nope", 1).is_err());
        // 2026-10-01 is a Thursday
        let d = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        assert_eq!(week_start(d).to_string(), "2026-09-28");
        assert_eq!(week_start(week_start(d)).to_string(), "2026-09-28");
    }

    #[test]
    fn invalid_timezone_is_rejected() {
        assert!(parse_timezone("Mars/Base").is_none());
        assert_eq!(
            parse_timezone(" Europe/London ").unwrap().name(),
            "Europe/London"
        );
    }
}
