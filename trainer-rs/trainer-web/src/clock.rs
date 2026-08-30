//! Local wall-clock time from the browser.
//!
//! `chrono`'s `Local` is unavailable here: its `wasmbind` feature would pull
//! `js-sys` and `wasm-bindgen` into `trainer-core`, which is deliberately free
//! of browser dependencies. The clock is a browser concern, so it lives here.

use chrono::{FixedOffset, NaiveDate, NaiveDateTime};
use trainer_core::datetime::TrainerTime;
use trainer_core::services::active_time::ActiveTime;

/// The current local wall-clock time, matching what `DateTime.Now` gave the C#.
///
/// Uses `Date`'s local getters rather than converting from UTC, so the browser
/// applies its own timezone and DST rules exactly as the .NET runtime did.
pub fn now_local() -> NaiveDateTime {
    let date = js_sys::Date::new_0();
    NaiveDate::from_ymd_opt(
        date.get_full_year() as i32,
        date.get_month() + 1, // JS months are zero-based
        date.get_date(),
    )
    .and_then(|d| {
        d.and_hms_milli_opt(
            date.get_hours(),
            date.get_minutes(),
            date.get_seconds(),
            date.get_milliseconds(),
        )
    })
    .unwrap_or_default()
}

/// The browser's current UTC offset.
///
/// `getTimezoneOffset` reports minutes *behind* UTC, so the sign is inverted.
fn browser_offset() -> Option<FixedOffset> {
    let minutes = -(js_sys::Date::new_0().get_timezone_offset() as i32);
    FixedOffset::east_opt(minutes * 60)
}

/// A wall-clock time stamped with the browser's offset, as `System.Text.Json`
/// stamped every `DateTime` whose kind was `Local` or `Unspecified`.
pub fn local_when(naive: NaiveDateTime) -> TrainerTime {
    match browser_offset() {
        Some(offset) => TrainerTime::Offset { naive, offset },
        // Only reachable for an offset outside +/-24h, which no browser reports.
        None => TrainerTime::Utc(naive),
    }
}

/// `DateTime.Now` as the activity form's `When` field holds it.
pub fn now_when() -> TrainerTime {
    local_when(now_local())
}

/// The current UTC time, matching `DateTime.UtcNow`.
///
/// Only the export stamp needs this: `ExportImportService` writes `exportDate`
/// in UTC while the export's *file name* uses local time, and reproducing that
/// split matters because the two differ by a day either side of midnight.
pub fn now_utc() -> NaiveDateTime {
    let date = js_sys::Date::new_0();
    NaiveDate::from_ymd_opt(
        date.get_utc_full_year() as i32,
        date.get_utc_month() + 1,
        date.get_utc_date(),
    )
    .and_then(|d| {
        d.and_hms_milli_opt(
            date.get_utc_hours(),
            date.get_utc_minutes(),
            date.get_utc_seconds(),
            date.get_utc_milliseconds(),
        )
    })
    .unwrap_or_default()
}

/// The current local time as an [`ActiveTime`], carrying the browser's UTC
/// offset so it serializes the way `DateTime.Now` did.
pub fn now_active_time() -> ActiveTime {
    let naive = now_local();
    match browser_offset() {
        Some(offset) => ActiveTime::Offset { naive, offset },
        None => ActiveTime::Unspecified(naive),
    }
}
