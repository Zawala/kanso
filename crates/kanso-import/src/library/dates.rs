//! Dates as note apps write them, to milliseconds since the epoch.

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone as _};

/// Reads a date or a date and time: RFC 3339 (`2024-03-01T09:30:00Z`,
/// `+02:00`), ISO 8601 without a zone (taken as local time), with a space
/// for the `T`, Evernote's `20240301T093000Z`, or a Unix timestamp in
/// seconds or milliseconds. `None` for anything else.
pub fn parse(text: &str) -> Option<i64> {
    let text = text.trim().trim_matches(['"', '\'']);
    if text.is_empty() {
        return None;
    }
    if text.bytes().all(|b| b.is_ascii_digit()) && text.len() >= 9 {
        let n: i64 = text.parse().ok()?;
        // Seconds until the year 5138; milliseconds after.
        return Some(if n < 100_000_000_000 { n * 1000 } else { n });
    }
    if let Ok(at) = DateTime::parse_from_rfc3339(text) {
        return Some(at.timestamp_millis());
    }
    for pattern in [
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M:%S %z",
    ] {
        if let Ok(at) = DateTime::parse_from_str(text, pattern) {
            return Some(at.timestamp_millis());
        }
    }
    if let Some(utc) = text.strip_suffix('Z') {
        for pattern in [
            "%Y%m%dT%H%M%S",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S%.f",
        ] {
            if let Ok(at) = NaiveDateTime::parse_from_str(utc, pattern) {
                return Some(at.and_utc().timestamp_millis());
            }
        }
    }
    for pattern in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
    ] {
        if let Ok(at) = NaiveDateTime::parse_from_str(text, pattern) {
            return local(at);
        }
    }
    for pattern in ["%Y-%m-%d", "%Y/%m/%d", "%Y_%m_%d", "%Y%m%d"] {
        if let Ok(date) = NaiveDate::parse_from_str(text, pattern) {
            return local(date.and_hms_opt(0, 0, 0)?);
        }
    }
    None
}

/// A date and time without a zone, read as local time.
pub fn local(at: NaiveDateTime) -> Option<i64> {
    Local
        .from_local_datetime(&at)
        .earliest()
        .map(|t| t.timestamp_millis())
        .or_else(|| Some(at.and_utc().timestamp_millis()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoned_forms() {
        assert_eq!(parse("2024-03-01T09:30:00Z"), Some(1_709_285_400_000));
        assert_eq!(parse("2024-03-01T11:30:00+02:00"), Some(1_709_285_400_000));
        assert_eq!(parse("20240301T093000Z"), Some(1_709_285_400_000));
        assert_eq!(parse("2024-03-01T09:30:00.000Z"), Some(1_709_285_400_000));
        assert_eq!(parse("1709285400"), Some(1_709_285_400_000));
        assert_eq!(parse("1709285400000"), Some(1_709_285_400_000));
    }

    #[test]
    fn local_forms_parse() {
        for text in [
            "2024-03-01",
            "2024-03-01 09:30",
            "2024-03-01T09:30:00",
            "2024/03/01",
        ] {
            assert!(parse(text).is_some(), "{text}");
        }
        for text in ["", "someday", "2024-13-01", "12"] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
