//! Relative time formatting, ported from `Trainer/Services/DateTimeHelper.cs`.
//!
//! The C# formats with `CultureInfo.InvariantCulture` throughout, so the output
//! is locale-independent and `chrono`'s English month and meridiem names match
//! it directly.

use crate::models::DurationOption;
use chrono::{Datelike, Days, NaiveDateTime, TimeDelta, Timelike};

/// `MMM d @ h:mm tt`, uppercased — for example `Jan 13 @ 10:22 AM`.
const DATE_AND_TIME: &str = "%b %-d @ %-I:%M %p";
/// `h:mm tt`, uppercased — for example `3:42 PM`.
const TIME_ONLY: &str = "%-I:%M %p";

/// `m:ss` for a running timer.
///
/// The C# takes `(int)Math.Max(0, elapsed.TotalSeconds)`, so a negative elapsed
/// time clamps to zero rather than rendering a negative clock.
pub fn format_elapsed(elapsed: TimeDelta) -> String {
    let total_seconds = elapsed.num_seconds().max(0);
    format!("{}:{:02}", total_seconds / 60, total_seconds % 60)
}

/// Formats a timestamp relative to `now`:
///
/// | age | output |
/// |---|---|
/// | future | `Jan 20 @ 10:00 AM` |
/// | under 2 hours | `15 minutes ago` |
/// | same day | `11:30 AM` |
/// | yesterday | `yesterday @ 8:30 AM` |
/// | older | `Jan 13 @ 10:22 AM` |
pub fn format_when(when: NaiveDateTime, now: NaiveDateTime) -> String {
    if when > now {
        return when.format(DATE_AND_TIME).to_string();
    }

    // `(int)timeDiff.TotalMinutes` truncates toward zero, as does num_minutes.
    let elapsed = now - when;
    if elapsed.num_minutes() < 120 {
        let minutes = elapsed.num_minutes();
        return if minutes <= 1 {
            "1 minute ago".to_owned()
        } else {
            format!("{minutes} minutes ago")
        };
    }

    if when.date() == now.date() {
        return when.format(TIME_ONLY).to_string();
    }

    if Some(when.date()) == now.date().checked_sub_days(Days::new(1)) {
        return format!("yesterday @ {}", when.format(TIME_ONLY));
    }

    when.format(DATE_AND_TIME).to_string()
}

/// The date range a filter selection covers.
///
/// `Week` runs from the Monday of the current week at midnight through Sunday
/// at 23:59:59; the others are simple offsets back from `now`.
pub fn date_range(duration: DurationOption, now: NaiveDateTime) -> (NaiveDateTime, NaiveDateTime) {
    match duration {
        DurationOption::Last24Hours => (now - TimeDelta::hours(24), now),
        DurationOption::Last7Days => (now - TimeDelta::days(7), now),
        DurationOption::Last4Weeks => (now - TimeDelta::days(28), now),
        DurationOption::Week => {
            let days_to_monday = now.date().weekday().num_days_from_monday() as i64;
            let week_start = (now.date() - TimeDelta::days(days_to_monday))
                .and_hms_opt(0, 0, 0)
                .expect("midnight is valid");
            let week_end = (week_start.date() + Days::new(6))
                .and_hms_opt(23, 59, 59)
                .expect("23:59:59 is valid");
            (week_start, week_end)
        }
    }
}

/// Whether a timestamp falls in the AM half of the day. Exposed because the
/// meridiem is the part most likely to drift if the format strings change.
pub fn is_morning(when: NaiveDateTime) -> bool {
    when.hour() < 12
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .expect("valid date")
            .and_hms_opt(h, min, s)
            .expect("valid time")
    }

    fn now() -> NaiveDateTime {
        dt(2025, 1, 15, 14, 30, 0)
    }

    // ── FormatWhenDateTime_RecentTimes_ReturnsMinutesAgo ──────────────────

    #[test]
    fn recent_times_render_as_minutes_ago() {
        for (minutes_ago, seconds_ago, expected) in [
            (0, 15, "1 minute ago"),
            (1, 0, "1 minute ago"),
            (15, 0, "15 minutes ago"),
            (119, 0, "119 minutes ago"),
            (119, 59, "119 minutes ago"),
        ] {
            let when = now() - TimeDelta::minutes(minutes_ago) - TimeDelta::seconds(seconds_ago);
            assert_eq!(
                format_when(when, now()),
                expected,
                "{minutes_ago}m {seconds_ago}s"
            );
        }
    }

    // ── FormatWhenDateTime_SameDayOlderThan2Hours_ReturnsTimeOnly ─────────

    #[test]
    fn same_day_beyond_two_hours_renders_time_only() {
        for (hour, minute, expected) in [
            (12, 30, "12:30 PM"),
            (11, 30, "11:30 AM"),
            (2, 25, "2:25 AM"),
            (0, 15, "12:15 AM"),
        ] {
            let when = dt(2025, 1, 15, hour, minute, 0);
            assert_eq!(format_when(when, now()), expected);
        }
    }

    // ── FormatWhenDateTime_Yesterday_ReturnsYesterdayAtTime ───────────────

    #[test]
    fn yesterday_is_labelled() {
        for (hour, minute, expected) in [
            (8, 30, "yesterday @ 8:30 AM"),
            (15, 42, "yesterday @ 3:42 PM"),
            (2, 25, "yesterday @ 2:25 AM"),
            (0, 0, "yesterday @ 12:00 AM"),
        ] {
            let when = dt(2025, 1, 14, hour, minute, 0);
            assert_eq!(format_when(when, now()), expected);
        }
    }

    // ── FormatWhenDateTime_OtherDates_ReturnsShortDateAndTime ─────────────

    #[test]
    fn older_and_future_dates_render_date_and_time() {
        for (y, m, d, h, min, expected) in [
            (2025, 1, 13, 10, 22, "Jan 13 @ 10:22 AM"),
            (2025, 1, 8, 9, 15, "Jan 8 @ 9:15 AM"),
            (2024, 12, 15, 16, 45, "Dec 15 @ 4:45 PM"),
            (2024, 1, 10, 10, 22, "Jan 10 @ 10:22 AM"),
            (2025, 1, 20, 10, 0, "Jan 20 @ 10:00 AM"),
        ] {
            assert_eq!(format_when(dt(y, m, d, h, min, 0), now()), expected);
        }
    }

    #[test]
    fn day_of_month_is_not_zero_padded() {
        assert_eq!(
            format_when(dt(2024, 3, 5, 9, 5, 0), now()),
            "Mar 5 @ 9:05 AM"
        );
    }

    // ── GetDateRange ──────────────────────────────────────────────────────

    #[test]
    fn relative_durations_offset_from_now() {
        for (duration, days) in [
            (DurationOption::Last24Hours, 1),
            (DurationOption::Last7Days, 7),
            (DurationOption::Last4Weeks, 28),
        ] {
            let (start, end) = date_range(duration, now());
            assert_eq!(start, now() - TimeDelta::days(days));
            assert_eq!(end, now());
        }
    }

    #[test]
    fn week_runs_monday_midnight_to_sunday_end_of_day() {
        // 15 January 2025 is a Wednesday.
        let (start, end) = date_range(DurationOption::Week, now());
        assert_eq!(start, dt(2025, 1, 13, 0, 0, 0));
        assert_eq!(end, dt(2025, 1, 19, 23, 59, 59));
    }

    #[test]
    fn week_starting_on_a_monday_does_not_move_back_a_week() {
        let monday = dt(2025, 1, 13, 9, 0, 0);
        let (start, _) = date_range(DurationOption::Week, monday);
        assert_eq!(start, dt(2025, 1, 13, 0, 0, 0));
    }

    #[test]
    fn week_on_a_sunday_starts_at_the_preceding_monday() {
        let sunday = dt(2025, 1, 19, 23, 0, 0);
        let (start, end) = date_range(DurationOption::Week, sunday);
        assert_eq!(start, dt(2025, 1, 13, 0, 0, 0));
        assert_eq!(end, dt(2025, 1, 19, 23, 59, 59));
    }

    // ── FormatElapsed ─────────────────────────────────────────────────────

    #[test]
    fn elapsed_renders_as_minutes_and_padded_seconds() {
        for (seconds, expected) in [
            (0, "0:00"),
            (1, "0:01"),
            (59, "0:59"),
            (60, "1:00"),
            (165, "2:45"),
            (999 * 60 + 59, "999:59"),
            (1000 * 60, "1000:00"),
        ] {
            assert_eq!(format_elapsed(TimeDelta::seconds(seconds)), expected);
        }
    }

    #[test]
    fn negative_elapsed_clamps_to_zero() {
        assert_eq!(format_elapsed(TimeDelta::seconds(-5)), "0:00");
    }

    #[test]
    fn is_morning_splits_at_noon() {
        assert!(is_morning(dt(2025, 1, 15, 11, 59, 0)));
        assert!(!is_morning(dt(2025, 1, 15, 12, 0, 0)));
    }
}
