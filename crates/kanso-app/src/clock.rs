//! The time the views show. The app reads the system clock and time zone;
//! the PNG snapshots fix both so that "Today" and "09:41" render the same on
//! every run.

use chrono::{Local, TimeZone as _};
use gpui_kit::{App, Global};

/// Set as a global to fix the time; without one, the system clock is used.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    /// Milliseconds since the epoch and the offset from UTC in seconds.
    fixed: Option<(i64, i32)>,
}

impl Global for Clock {}

impl Clock {
    /// A clock stopped at `now_ms`, in a zone `utc_offset_secs` from UTC.
    #[cfg(test)]
    pub fn fixed(now_ms: i64, utc_offset_secs: i32) -> Self {
        Clock {
            fixed: Some((now_ms, utc_offset_secs)),
        }
    }

    fn fixed_in(cx: &App) -> Option<(i64, i32)> {
        cx.try_global::<Clock>().and_then(|clock| clock.fixed)
    }

    /// Now, in milliseconds since the epoch.
    pub fn now_ms(cx: &App) -> i64 {
        Self::fixed_in(cx).map_or_else(kanso_core::now_ms, |(now, _)| now)
    }

    /// How far local time is ahead of UTC at the instant `at_ms`, in seconds.
    /// It can differ between two instants across a daylight-saving change.
    pub fn utc_offset_secs(cx: &App, at_ms: i64) -> i32 {
        match Self::fixed_in(cx) {
            Some((_, offset)) => offset,
            None => Local
                .timestamp_millis_opt(at_ms)
                .earliest()
                .map_or(0, |time| time.offset().local_minus_utc()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Clock;
    use gpui_kit::TestAppContext;

    // no-prop-test: reads a global or the system clock; nothing to vary.
    #[gpui_kit::test]
    fn a_fixed_clock_wins_over_the_system_one(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let before = kanso_core::now_ms();
            let now = Clock::now_ms(cx);
            assert!(now >= before && now - before < 60_000);
            // Any real zone is within a day of UTC.
            assert!(Clock::utc_offset_secs(cx, now).abs() <= 86_400);

            cx.set_global(Clock::fixed(1_000, -3_600));
            assert_eq!(Clock::now_ms(cx), 1_000);
            assert_eq!(Clock::utc_offset_secs(cx, 5), -3_600);
        });
    }
}
