//! Local wall-clock time from the browser.
//!
//! `chrono`'s `Local` is unavailable here: its `wasmbind` feature would pull
//! `js-sys` and `wasm-bindgen` into `trainer-core`, which is deliberately free
//! of browser dependencies. The clock is a browser concern, so it lives here.

use chrono::{NaiveDate, NaiveDateTime};
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

/// The current local time as an [`ActiveTime`], carrying the browser's UTC
/// offset so it serializes the way `DateTime.Now` did.
pub fn now_active_time() -> ActiveTime {
    let naive = now_local();
    // getTimezoneOffset is minutes *behind* UTC, so the sign is inverted.
    let offset_minutes = -(js_sys::Date::new_0().get_timezone_offset() as i32);
    match chrono::FixedOffset::east_opt(offset_minutes * 60) {
        Some(offset) => ActiveTime::Offset { naive, offset },
        None => ActiveTime::Unspecified(naive),
    }
}
