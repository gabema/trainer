//! Week bucketing, ported from `Trainer/Services/WeekHelper.cs`.
//!
//! # This is not ISO 8601, despite the C# doc comment saying so
//!
//! `GetWeekKey` pairs the **plain calendar year** with a week number computed
//! under .NET's `CalendarWeekRule.FirstFourDayWeek` starting on Monday. Real
//! ISO 8601 uses the *ISO week-year*, which diverges at year boundaries: .NET
//! returns week 53 for an early-January date while `GetYear` returns the new
//! calendar year, producing keys such as `2010.53` for 1 January 2010 where ISO
//! numbering would say `2009-W53`.
//!
//! `chrono::IsoWeek` returns correct ISO week-years and would therefore
//! **silently re-bucket every activity near a year boundary** into storage keys
//! that nothing reads. It is deliberately not used here.
//!
//! The consequence is visible but harmless: the calendar week straddling New
//! Year splits across two buckets — `2025.53` holds 29–31 December and `2026.01`
//! holds 1–4 January. Storage stays self-consistent because reads and writes
//! both go through [`week_key`].
//!
//! Verified against `tests/fixtures/week-keys.csv`, 11,323 days of golden values
//! generated from the C# implementation.

use chrono::{Datelike, Days, NaiveDate, NaiveDateTime};
use std::fmt;

/// `DayOfWeek.Monday`, in .NET's numbering where Sunday is 0.
const FIRST_DAY_OF_WEEK: i32 = 1;
/// `CalendarWeekRule.FirstFourDayWeek`.
const FULL_DAYS: i32 = 4;

const STORAGE_PREFIX: &str = "activities-";

/// A malformed week key or an out-of-range year.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekKeyError(pub String);

impl fmt::Display for WeekKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid week key format: {}", self.0)
    }
}

impl std::error::Error for WeekKeyError {}

/// .NET's `DayOfWeek`: Sunday = 0 through Saturday = 6.
fn dotnet_day_of_week(date: NaiveDate) -> i32 {
    date.weekday().num_days_from_sunday() as i32
}

/// Reproduces .NET's `GetWeekOfYearFullDays` for `FirstFourDayWeek` / Monday.
///
/// When the computed day falls before the start of the first week, .NET recurses
/// onto 31 December of the previous year, which is how an early-January date
/// ends up reported as week 52 or 53.
fn week_of_year(date: NaiveDate) -> i32 {
    let day_of_year = date.ordinal() as i32 - 1;
    let day_for_jan1 = dotnet_day_of_week(date) - (day_of_year % 7);

    let mut offset = (FIRST_DAY_OF_WEEK - day_for_jan1 + 14) % 7;
    if offset != 0 && offset >= FULL_DAYS {
        offset -= 7;
    }

    let day = day_of_year - offset;
    if day >= 0 {
        return day / 7 + 1;
    }

    let december_31 = date - Days::new(day_of_year as u64 + 1);
    week_of_year(december_31)
}

/// The storage bucket key for a date, formatted `YYYY.WW`.
pub fn week_key_for_date(date: NaiveDate) -> String {
    format!("{}.{:02}", date.year(), week_of_year(date))
}

/// The storage bucket key for a timestamp. Only the date part participates.
pub fn week_key(when: NaiveDateTime) -> String {
    week_key_for_date(when.date())
}

fn parse_week_key(key: &str) -> Result<(i32, i32), WeekKeyError> {
    let mut parts = key.split('.');
    let (Some(year), Some(week), None) = (parts.next(), parts.next(), parts.next()) else {
        return Err(WeekKeyError(key.to_owned()));
    };

    let year: i32 = year.parse().map_err(|_| WeekKeyError(key.to_owned()))?;
    let week: i32 = week.parse().map_err(|_| WeekKeyError(key.to_owned()))?;
    Ok((year, week))
}

/// The Monday of the week a key names.
///
/// **Ported as-is, including a defect.** The scan finds the first date in the
/// key's calendar year that produces the key, then walks back to that week's
/// Monday — which for the first week of any year lies in the *previous* year,
/// and therefore in a different bucket:
///
/// ```text
/// week_start_date("2026.01") -> 2025-12-29   (bucket 2025.53, not 2026.01)
/// ```
///
/// This happens in every year whose 1 January is **not** a Monday — 27 of the
/// 31 years from 2010 to 2040, the exceptions being 2018, 2024, 2029 and 2035.
/// When New Year's Day is itself a Monday the walk-back is zero days and the key
/// round-trips. See `tests/fixtures/week-key-anomalies.csv`.
///
/// Combined with `GetAllAsync` performing no date filtering, it lets the
/// Activities and Calendar views load a bucket twice. Nothing is persisted from
/// this function, so correcting it is data-safe — but it is a user-visible
/// behavior change and belongs in its own change, not smuggled into a port.
///
/// When no date in the year produces the key, the scan leaves the cursor at
/// 1 January and returns that week's Monday, pinned in
/// `tests/fixtures/week-unmatched-keys.csv`.
pub fn week_start_date(key: &str) -> Result<NaiveDate, WeekKeyError> {
    let (year, _week) = parse_week_key(key)?;

    let january_1 =
        NaiveDate::from_ymd_opt(year, 1, 1).ok_or_else(|| WeekKeyError(key.to_owned()))?;
    let december_31 =
        NaiveDate::from_ymd_opt(year, 12, 31).ok_or_else(|| WeekKeyError(key.to_owned()))?;

    // Linear scan, as the C# does. O(365) per call, and the fallback to
    // 1 January when nothing matches is observable behavior.
    let mut date_in_week = january_1;
    let mut cursor = january_1;
    while cursor <= december_31 {
        if week_key_for_date(cursor) == key {
            date_in_week = cursor;
            break;
        }
        match cursor.succ_opt() {
            Some(next) => cursor = next,
            None => break,
        }
    }

    let days_to_monday = (dotnet_day_of_week(date_in_week) - FIRST_DAY_OF_WEEK + 7) % 7;
    Ok(date_in_week - Days::new(days_to_monday as u64))
}

/// The last moment of the week a key names: the Sunday at 23:59:59.
pub fn week_end_date(key: &str) -> Result<NaiveDateTime, WeekKeyError> {
    let start = week_start_date(key)?;
    (start + Days::new(6))
        .and_hms_opt(23, 59, 59)
        .ok_or_else(|| WeekKeyError(key.to_owned()))
}

/// Every distinct week key touched by a date range, in first-seen order.
///
/// Iterates day by day rather than week by week so that a range spanning a year
/// boundary cannot skip a bucket — 31 December 2025 to 5 January 2026 covers
/// both `2025.53` and `2026.01`.
pub fn week_keys_in_range(start: NaiveDateTime, end: NaiveDateTime) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut keys = Vec::new();

    let mut cursor = start.date();
    let last = end.date();
    while cursor <= last {
        let key = week_key_for_date(cursor);
        if seen.insert(key.clone()) {
            keys.push(key);
        }
        match cursor.succ_opt() {
            Some(next) => cursor = next,
            None => break,
        }
    }

    keys
}

/// `activities-YYYY.WW`.
pub fn storage_key(week_key: &str) -> String {
    format!("{STORAGE_PREFIX}{week_key}")
}

/// Recovers `YYYY.WW` from `activities-YYYY.WW`.
pub fn extract_week_key(storage_key: &str) -> Result<&str, WeekKeyError> {
    storage_key
        .strip_prefix(STORAGE_PREFIX)
        .ok_or_else(|| WeekKeyError(storage_key.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::read_fixture;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// Task 5.2 — the whole point of the golden fixture.
    #[test]
    fn every_golden_week_key_reproduces() {
        let csv = read_fixture("week-keys.csv");
        let mut lines = csv.lines();
        assert_eq!(lines.next(), Some("date,weekKey"));

        let mut checked = 0;
        for line in lines {
            let (raw_date, expected) = line.split_once(',').expect("two columns");
            let parsed = NaiveDate::parse_from_str(raw_date, "%Y-%m-%d").expect("ISO date");
            assert_eq!(
                week_key_for_date(parsed),
                expected,
                "week key diverged on {raw_date}"
            );
            checked += 1;
        }

        assert_eq!(checked, 11323, "expected every day from 2010 through 2040");
    }

    #[test]
    fn diverges_from_iso_8601_at_year_boundaries() {
        // The cases chrono::IsoWeek would get "right" and thereby re-bucket.
        assert_eq!(week_key_for_date(date(2010, 1, 1)), "2010.53");
        assert_eq!(week_key_for_date(date(2011, 1, 1)), "2011.52");
        assert_eq!(week_key_for_date(date(2016, 1, 1)), "2016.53");
        assert_eq!(week_key_for_date(date(2021, 1, 1)), "2021.53");

        // The New Year week splits across two buckets rather than colliding.
        assert_eq!(week_key_for_date(date(2025, 12, 29)), "2025.53");
        assert_eq!(week_key_for_date(date(2025, 12, 31)), "2025.53");
        assert_eq!(week_key_for_date(date(2026, 1, 1)), "2026.01");
        assert_eq!(week_key_for_date(date(2026, 1, 4)), "2026.01");
        assert_eq!(week_key_for_date(date(2026, 1, 5)), "2026.02");
    }

    /// Task 5.3 / 5.4 — golden start and end dates for every observed key.
    #[test]
    fn every_golden_week_boundary_reproduces() {
        let csv = read_fixture("week-boundaries.csv");
        let mut lines = csv.lines();
        assert_eq!(lines.next(), Some("weekKey,startDate,endDate"));

        let mut checked = 0;
        for line in lines {
            let cols: Vec<&str> = line.split(',').collect();
            assert_eq!(cols.len(), 3, "malformed row: {line}");

            let expected_start =
                NaiveDateTime::parse_from_str(cols[1], "%Y-%m-%dT%H:%M:%S").expect("start parses");
            let expected_end =
                NaiveDateTime::parse_from_str(cols[2], "%Y-%m-%dT%H:%M:%S").expect("end parses");

            assert_eq!(
                week_start_date(cols[0]).expect("resolves"),
                expected_start.date(),
                "start date diverged for {}",
                cols[0]
            );
            assert_eq!(
                week_end_date(cols[0]).expect("resolves"),
                expected_end,
                "end date diverged for {}",
                cols[0]
            );
            checked += 1;
        }

        assert!(checked > 1500, "only {checked} week keys checked");
    }

    /// The defect this port deliberately reproduces.
    #[test]
    fn week_start_date_lands_outside_its_own_bucket_at_year_boundaries() {
        let csv = read_fixture("week-key-anomalies.csv");
        let mut lines = csv.lines();
        lines.next();

        let mut checked = 0;
        for line in lines {
            let cols: Vec<&str> = line.split(',').collect();
            let (key, expected_start, expected_resolved) = (cols[0], cols[2], cols[3]);

            let start = week_start_date(key).expect("resolves");
            assert_eq!(start.format("%Y-%m-%d").to_string(), expected_start);

            // The round trip fails: the Monday belongs to a different bucket.
            assert_eq!(week_key_for_date(start), expected_resolved);
            assert_ne!(week_key_for_date(start), key);
            checked += 1;
        }

        assert_eq!(checked, 27, "anomalies recorded for 2010 through 2040");
    }

    /// The anomaly is not arbitrary: the first week key of a year fails to round
    /// trip exactly when 1 January is not a Monday, because walking back to that
    /// week's Monday then crosses into the previous year. In 2018, 2024, 2029 and
    /// 2035 New Year's Day *is* a Monday, so those years are clean.
    #[test]
    fn the_anomaly_occurs_exactly_when_january_first_is_not_a_monday() {
        use chrono::Weekday;

        let mut anomalies = 0;
        for year in 2010..=2040 {
            let january_1 = date(year, 1, 1);
            let key = week_key_for_date(january_1);
            let start = week_start_date(&key).expect("resolves");
            let round_trips = week_key_for_date(start) == key;

            assert_eq!(
                round_trips,
                january_1.weekday() == Weekday::Mon,
                "year {year}: round-trip should hold only when 1 January is a Monday"
            );

            if !round_trips {
                anomalies += 1;
            }
        }

        assert_eq!(anomalies, 27);
    }

    /// Task 5.3's fallback branch, which no other fixture reaches.
    #[test]
    fn unmatched_week_keys_fall_back_to_the_january_first_week() {
        let csv = read_fixture("week-unmatched-keys.csv");
        let mut lines = csv.lines();
        lines.next();

        let mut unmatched = 0;
        for line in lines {
            let cols: Vec<&str> = line.split(',').collect();
            let (key, expected_start, expected_end, produced) =
                (cols[0], cols[1], cols[2], cols[3]);

            let start = week_start_date(key).expect("resolves");
            let end = week_end_date(key).expect("resolves");

            assert_eq!(
                start,
                NaiveDateTime::parse_from_str(expected_start, "%Y-%m-%dT%H:%M:%S")
                    .expect("parses")
                    .date(),
                "fallback start diverged for {key}"
            );
            assert_eq!(
                end,
                NaiveDateTime::parse_from_str(expected_end, "%Y-%m-%dT%H:%M:%S").expect("parses"),
                "fallback end diverged for {key}"
            );

            if produced == "false" {
                unmatched += 1;
            }
        }

        assert!(
            unmatched >= 4,
            "the fallback branch must actually be exercised"
        );
    }

    #[test]
    fn week_keys_in_range_spans_the_year_boundary() {
        let keys = week_keys_in_range(
            date(2025, 12, 31).and_hms_opt(9, 0, 0).expect("valid"),
            date(2026, 1, 5).and_hms_opt(18, 0, 0).expect("valid"),
        );
        assert_eq!(keys, vec!["2025.53", "2026.01", "2026.02"]);
    }

    #[test]
    fn week_keys_in_range_is_first_seen_order_and_deduplicated() {
        let keys = week_keys_in_range(
            date(2026, 1, 1).and_hms_opt(0, 0, 0).expect("valid"),
            date(2026, 1, 3).and_hms_opt(23, 59, 59).expect("valid"),
        );
        assert_eq!(keys, vec!["2026.01"]);
    }

    #[test]
    fn week_keys_in_range_handles_an_inverted_range() {
        let keys = week_keys_in_range(
            date(2026, 6, 1).and_hms_opt(0, 0, 0).expect("valid"),
            date(2026, 1, 1).and_hms_opt(0, 0, 0).expect("valid"),
        );
        assert!(keys.is_empty());
    }

    #[test]
    fn storage_keys_round_trip() {
        assert_eq!(storage_key("2026.01"), "activities-2026.01");
        assert_eq!(
            extract_week_key("activities-2026.01").expect("valid"),
            "2026.01"
        );
        assert!(extract_week_key("activityTypes").is_err());
        assert!(extract_week_key("2026.01").is_err());
    }

    #[test]
    fn malformed_week_keys_are_rejected() {
        for key in ["", "2026", "2026.01.02", "abc.01", "2026.xx", "."] {
            assert!(week_start_date(key).is_err(), "{key:?} should be rejected");
        }
    }

    #[test]
    fn out_of_range_year_is_rejected_rather_than_panicking() {
        assert!(week_start_date("999999.01").is_err());
        assert!(week_start_date("-5000000.01").is_err());
    }
}
