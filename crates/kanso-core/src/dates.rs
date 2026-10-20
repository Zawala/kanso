//! Calendar dates and times without a time-zone library: days counted from
//! the Unix epoch in the proleptic Gregorian calendar, and a small
//! `strftime`-like formatter for templates (`{{date:%A %e %B}}`).
//!
//! # Format directives
//!
//! | Directive | Meaning                         | Example     |
//! |-----------|---------------------------------|-------------|
//! | `%Y`      | Year, four digits               | `2026`      |
//! | `%y`      | Year, last two digits           | `26`        |
//! | `%m`      | Month, `01`–`12`                | `10`        |
//! | `%d`      | Day of the month, `01`–`31`     | `07`        |
//! | `%e`      | Day of the month, no padding    | `7`         |
//! | `%B`      | Month name                      | `October`   |
//! | `%b`      | Month name, three letters       | `Oct`       |
//! | `%A`      | Weekday name                    | `Tuesday`   |
//! | `%a`      | Weekday name, three letters     | `Tue`       |
//! | `%j`      | Day of the year, `001`–`366`    | `293`       |
//! | `%u`      | Weekday number, Monday = 1      | `2`         |
//! | `%V`      | ISO 8601 week number, `01`–`53` | `43`        |
//! | `%H`      | Hour, `00`–`23`                 | `09`        |
//! | `%I`      | Hour, `01`–`12`                 | `09`        |
//! | `%M`      | Minute, `00`–`59`               | `05`        |
//! | `%S`      | Second, `00`–`59`               | `00`        |
//! | `%p`      | `AM` or `PM`                    | `AM`        |
//! | `%%`      | A percent sign                  | `%`         |
//!
//! Anything else is copied as it is, so `%Q` stays `%Q`. Names are English.

use std::fmt::Write as _;

const DAY_MS: i64 = 86_400_000;

pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Monday first, as ISO 8601 counts them.
pub const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// A day in the proleptic Gregorian calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    /// 1–31.
    pub day: u32,
}

/// A local date and time of day, to the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    pub date: Date,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

impl Date {
    /// The date, when it exists.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Date { year, month, day })
    }

    /// The day `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`).
    pub fn from_days(days: i64) -> Date {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = yoe + era * 400 + i64::from(month <= 2);
        Date {
            year: year as i32,
            month,
            day,
        }
    }

    /// Days since 1970-01-01 (`days_from_civil`).
    pub fn to_days(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let yoe = year.rem_euclid(400);
        let month = i64::from(self.month);
        let mp = if month > 2 { month - 3 } else { month + 9 };
        let doy = (153 * mp + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The day of a local time in milliseconds since the epoch (a UTC time
    /// with the zone's offset already added).
    pub fn from_local_ms(local_ms: i64) -> Date {
        Date::from_days(local_ms.div_euclid(DAY_MS))
    }

    pub fn add_days(self, days: i64) -> Date {
        Date::from_days(self.to_days() + days)
    }

    /// 0 for Monday through 6 for Sunday.
    pub fn weekday(self) -> usize {
        // 1970-01-01 was a Thursday.
        (self.to_days() + 3).rem_euclid(7) as usize
    }

    /// 1 for 1 January.
    pub fn ordinal(self) -> u32 {
        (self.to_days() - Date::new(self.year, 1, 1).unwrap_or(self).to_days()) as u32 + 1
    }

    /// The ISO 8601 week number.
    pub fn iso_week(self) -> u32 {
        // The week belongs to the year its Thursday is in.
        let thursday = self.add_days(3 - self.weekday() as i64);
        (thursday.ordinal() - 1) / 7 + 1
    }

    /// `YYYY-MM-DD`.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Reads `YYYY-MM-DD`, the date part of an ISO 8601 string.
    pub fn parse_iso(text: &str) -> Option<Date> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let digits = |range: std::ops::Range<usize>| -> Option<u32> {
            let part = text.get(range)?;
            part.bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| part.parse().ok())?
        };
        Date::new(digits(0..4)? as i32, digits(5..7)?, digits(8..10)?)
    }

    /// "Tuesday, 20 October 2026".
    pub fn long_label(self) -> String {
        format(
            &DateTime {
                date: self,
                hour: 0,
                minute: 0,
                second: 0,
            },
            "%A, %e %B %Y",
        )
    }
}

impl DateTime {
    /// The local date and time of a local time in milliseconds since the
    /// epoch (a UTC time with the zone's offset already added).
    pub fn from_local_ms(local_ms: i64) -> DateTime {
        let date = Date::from_local_ms(local_ms);
        let secs = local_ms.rem_euclid(DAY_MS) / 1000;
        DateTime {
            date,
            hour: (secs / 3600) as u32,
            minute: (secs / 60 % 60) as u32,
            second: (secs % 60) as u32,
        }
    }

    pub fn at_midnight(date: Date) -> DateTime {
        DateTime {
            date,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

/// `dt` written as `pattern` says; see the module's table of directives.
pub fn format(dt: &DateTime, pattern: &str) -> String {
    let d = dt.date;
    let mut out = String::with_capacity(pattern.len() + 16);
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let Some(directive) = chars.next() else {
            out.push('%');
            break;
        };
        let _ = match directive {
            'Y' => write!(out, "{:04}", d.year),
            'y' => write!(out, "{:02}", d.year.rem_euclid(100)),
            'm' => write!(out, "{:02}", d.month),
            'd' => write!(out, "{:02}", d.day),
            'e' => write!(out, "{}", d.day),
            'B' => write!(out, "{}", MONTHS[d.month as usize - 1]),
            'b' => write!(out, "{}", &MONTHS[d.month as usize - 1][..3]),
            'A' => write!(out, "{}", WEEKDAYS[d.weekday()]),
            'a' => write!(out, "{}", &WEEKDAYS[d.weekday()][..3]),
            'j' => write!(out, "{:03}", d.ordinal()),
            'u' => write!(out, "{}", d.weekday() + 1),
            'V' => write!(out, "{:02}", d.iso_week()),
            'H' => write!(out, "{:02}", dt.hour),
            'I' => write!(out, "{:02}", (dt.hour + 11) % 12 + 1),
            'M' => write!(out, "{:02}", dt.minute),
            'S' => write!(out, "{:02}", dt.second),
            'p' => write!(out, "{}", if dt.hour < 12 { "AM" } else { "PM" }),
            '%' => write!(out, "%"),
            other => write!(out, "%{other}"),
        };
    }
    out
}

/// Reads a date written with `pattern`, for the directives that name the
/// day: `%Y %m %d %e %B %b %A %a %j`, plus literal text. Other directives
/// can't be read back and give `None`. Names match without regard to case.
pub fn parse_date(text: &str, pattern: &str) -> Option<Date> {
    let mut rest = text;
    let (mut year, mut month, mut day, mut ordinal) = (None, None, None, None);
    let mut weekday = None;
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            rest = rest.strip_prefix(c)?;
            continue;
        }
        match chars.next()? {
            'Y' => {
                let (n, r) = take_digits(rest, 4, 4)?;
                year = Some(n as i32);
                rest = r;
            }
            'm' => {
                let (n, r) = take_digits(rest, 2, 2)?;
                month = Some(n);
                rest = r;
            }
            'd' => {
                let (n, r) = take_digits(rest, 2, 2)?;
                day = Some(n);
                rest = r;
            }
            'e' => {
                let (n, r) = take_digits(rest, 1, 2)?;
                day = Some(n);
                rest = r;
            }
            'j' => {
                let (n, r) = take_digits(rest, 3, 3)?;
                ordinal = Some(n);
                rest = r;
            }
            'B' => {
                let (ix, r) = take_name(rest, &MONTHS, None)?;
                month = Some(ix as u32 + 1);
                rest = r;
            }
            'b' => {
                let (ix, r) = take_name(rest, &MONTHS, Some(3))?;
                month = Some(ix as u32 + 1);
                rest = r;
            }
            'A' => {
                let (ix, r) = take_name(rest, &WEEKDAYS, None)?;
                weekday = Some(ix);
                rest = r;
            }
            'a' => {
                let (ix, r) = take_name(rest, &WEEKDAYS, Some(3))?;
                weekday = Some(ix);
                rest = r;
            }
            '%' => rest = rest.strip_prefix('%')?,
            _ => return None,
        }
    }
    if !rest.is_empty() {
        return None;
    }
    let year = year?;
    let date = match (month, day, ordinal) {
        (Some(month), Some(day), _) => Date::new(year, month, day)?,
        (_, _, Some(ordinal)) => {
            let first = Date::new(year, 1, 1)?;
            let date = first.add_days(i64::from(ordinal) - 1);
            (ordinal >= 1 && date.year == year).then_some(date)?
        }
        _ => return None,
    };
    if weekday.is_some_and(|w| w != date.weekday()) {
        return None;
    }
    Some(date)
}

/// `min..=max` ASCII digits from the start of `text`, greedily.
fn take_digits(text: &str, min: usize, max: usize) -> Option<(u32, &str)> {
    let len = text
        .bytes()
        .take(max)
        .take_while(u8::is_ascii_digit)
        .count();
    if len < min {
        return None;
    }
    Some((text[..len].parse().ok()?, &text[len..]))
}

/// The longest name in `names` (or its first `short` letters) that `text`
/// starts with, ignoring case.
fn take_name<'a>(text: &'a str, names: &[&str], short: Option<usize>) -> Option<(usize, &'a str)> {
    names
        .iter()
        .enumerate()
        .map(|(ix, name)| (ix, short.map_or(*name, |n| &name[..n])))
        .filter(|(_, name)| {
            text.get(..name.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(name))
        })
        .max_by_key(|(_, name)| name.len())
        .map(|(ix, name)| (ix, &text[name.len()..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime {
        DateTime {
            date: Date::new(y, m, d).unwrap(),
            hour: h,
            minute: min,
            second: 0,
        }
    }

    #[test]
    fn known_days() {
        assert_eq!(Date::from_days(0), Date::new(1970, 1, 1).unwrap());
        assert_eq!(Date::new(2026, 10, 20).unwrap().weekday(), 1); // Tuesday
        assert_eq!(Date::new(2000, 2, 29).unwrap().ordinal(), 60);
        assert_eq!(Date::new(2026, 1, 1).unwrap().iso_week(), 1);
        assert_eq!(Date::new(2027, 1, 1).unwrap().iso_week(), 53);
        assert_eq!(Date::new(2025, 12, 29).unwrap().iso_week(), 1);
        assert!(Date::new(2026, 2, 29).is_none());
        assert_eq!(
            Date::parse_iso("2026-10-20"),
            Date::new(2026, 10, 20),
            "ISO dates read back"
        );
        assert_eq!(Date::parse_iso("2026-13-01"), None);
        assert_eq!(Date::parse_iso("2026-1-01"), None);
    }

    #[test]
    fn format_examples() {
        let t = dt(2026, 10, 7, 21, 5);
        assert_eq!(format(&t, "%Y-%m-%d"), "2026-10-07");
        assert_eq!(format(&t, "%A, %e %B %Y"), "Wednesday, 7 October 2026");
        assert_eq!(format(&t, "%a %b %d %y"), "Wed Oct 07 26");
        assert_eq!(format(&t, "%H:%M %I%p"), "21:05 09PM");
        assert_eq!(format(&t, "%j %u W%V"), "280 3 W41");
        assert_eq!(format(&t, "100%% %Q %"), "100% %Q %");
        assert_eq!(
            dt(2026, 1, 1, 0, 0).date.long_label(),
            "Thursday, 1 January 2026"
        );
    }

    #[test]
    fn parse_examples() {
        let d = Date::new(2026, 10, 7);
        assert_eq!(parse_date("2026-10-07", "%Y-%m-%d"), d);
        assert_eq!(parse_date("wednesday 7 OCTOBER 2026", "%A %e %B %Y"), d);
        assert_eq!(parse_date("Thursday 7 October 2026", "%A %e %B %Y"), None);
        assert_eq!(parse_date("2026-280", "%Y-%j"), d);
        assert_eq!(parse_date("2026-10-07 x", "%Y-%m-%d"), None);
        assert_eq!(parse_date("21:05", "%H:%M"), None);
    }

    fn date() -> impl Strategy<Value = Date> {
        (-200_000i64..200_000).prop_map(Date::from_days)
    }

    /// Directives that name the day, with separators that can't be taken
    /// for part of a number or a name.
    fn date_pattern() -> impl Strategy<Value = String> {
        let token = prop_oneof![
            Just("%Y-%m-%d".to_owned()),
            Just("%Y%m%d".to_owned()),
            Just("%e %B %Y".to_owned()),
            Just("%A, %e %B %Y".to_owned()),
            Just("%a %b %d %Y".to_owned()),
            Just("%d/%m/%Y".to_owned()),
            Just("%Y-%j".to_owned()),
            Just("%B %e, %Y".to_owned()),
        ];
        (token, "[ .,:/_x]{0,3}").prop_map(|(t, sep)| format!("{sep}{t}{sep}"))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Round trip: days to a date and back.
        #[test]
        fn prop_days_round_trip(days in -1_000_000i64..1_000_000) {
            let d = Date::from_days(days);
            prop_assert_eq!(d.to_days(), days);
            prop_assert!(Date::new(d.year, d.month, d.day).is_some());
        }

        /// Invariant: consecutive days are consecutive weekdays and dates.
        #[test]
        fn prop_next_day_follows(d in date()) {
            let next = d.add_days(1);
            prop_assert!(next > d);
            prop_assert_eq!(next.weekday(), (d.weekday() + 1) % 7);
            prop_assert_eq!(next.add_days(-1), d);
        }

        /// Round trip: the ISO form reads back for four-digit years.
        #[test]
        fn prop_iso_round_trips(days in -700_000i64..2_900_000) {
            let d = Date::from_days(days);
            prop_assert_eq!(Date::parse_iso(&d.iso()), Some(d));
        }

        /// Round trip: a date written with any day-naming pattern reads
        /// back as the same date.
        #[test]
        fn prop_format_then_parse_round_trips(days in -700_000i64..2_900_000, pattern in date_pattern()) {
            let d = Date::from_days(days);
            let text = format(&DateTime::at_midnight(d), &pattern);
            prop_assert_eq!(parse_date(&text, &pattern), Some(d), "{}", text);
        }

        /// Invariant: formatting never panics and leaves text without
        /// directives as it is.
        #[test]
        fn prop_plain_text_formats_as_itself(text in "[^%]{0,40}", d in date()) {
            prop_assert_eq!(format(&DateTime::at_midnight(d), &text), text);
        }

        /// Round trip: a local time's date and time of day give it back.
        #[test]
        fn prop_local_ms_splits(ms in -10_000_000_000_000i64..10_000_000_000_000) {
            let t = DateTime::from_local_ms(ms);
            let back = t.date.to_days() * DAY_MS
                + i64::from(t.hour) * 3_600_000
                + i64::from(t.minute) * 60_000
                + i64::from(t.second) * 1000;
            prop_assert_eq!(back, ms - ms.rem_euclid(1000));
        }
    }
}
