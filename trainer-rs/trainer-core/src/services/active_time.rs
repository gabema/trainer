//! The timestamp format used by `ActiveActivityService`, which is **not** the
//! one activities use.
//!
//! That service persists with `JsonSerializer.Serialize(entries)` and no
//! options, so no `DateTimeConverter` applies and `System.Text.Json`'s default
//! `DateTime` handling takes over:
//!
//! | kind | activities (`DateTimeConverter`) | active activities (default) |
//! |---|---|---|
//! | `Local` | `2026-08-28T15:43:21-07` | `2026-08-28T15:43:21-07:00` |
//! | `Utc` | `2026-06-15T10:00:00Z` | same |
//! | `Unspecified` | a local offset is applied | `2026-01-01T00:00:00` |
//! | sub-second | never emitted | `.1234567`, `.1` |
//!
//! [`TrainerTime`](crate::datetime::TrainerTime) cannot serve this: its parser
//! is RFC 3339, which requires an offset and so rejects the third form, and it
//! never emits fractional seconds. Recorded in
//! `tests/fixtures/active-activities.json`.

use chrono::{FixedOffset, NaiveDateTime, Timelike};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

const BASE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
/// .NET ticks are 100 nanoseconds, so at most seven fractional digits.
const MAX_FRACTIONAL_DIGITS: u32 = 7;

/// A start time as `ActiveActivityService` stores one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTime {
    /// `DateTimeKind.Utc` — a trailing `Z`.
    Utc(NaiveDateTime),
    /// `DateTimeKind.Local` — a full `±hh:mm` offset, never the hour-only form.
    Offset {
        naive: NaiveDateTime,
        offset: FixedOffset,
    },
    /// `DateTimeKind.Unspecified` — no suffix at all.
    Unspecified(NaiveDateTime),
}

impl ActiveTime {
    pub fn naive(&self) -> NaiveDateTime {
        match self {
            ActiveTime::Utc(n) | ActiveTime::Unspecified(n) => *n,
            ActiveTime::Offset { naive, .. } => *naive,
        }
    }

    /// Fractional seconds with trailing zeros trimmed, empty when whole.
    fn fraction(naive: NaiveDateTime) -> String {
        let ticks = naive.nanosecond() / 100;
        if ticks == 0 {
            return String::new();
        }
        let digits = format!("{ticks:0width$}", width = MAX_FRACTIONAL_DIGITS as usize);
        let trimmed = digits.trim_end_matches('0');
        format!(".{trimmed}")
    }

    fn format_offset(offset: FixedOffset) -> String {
        let total_minutes = offset.local_minus_utc() / 60;
        let sign = if total_minutes < 0 { '-' } else { '+' };
        let hours = (total_minutes / 60).abs();
        let minutes = (total_minutes % 60).abs();
        format!("{sign}{hours:02}:{minutes:02}")
    }

    pub fn to_wire(&self) -> String {
        let naive = self.naive();
        let base = format!("{}{}", naive.format(BASE_FORMAT), Self::fraction(naive));
        match self {
            ActiveTime::Utc(_) => format!("{base}Z"),
            ActiveTime::Unspecified(_) => base,
            ActiveTime::Offset { offset, .. } => {
                format!("{base}{}", Self::format_offset(*offset))
            }
        }
    }

    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let error = || ParseError(text.to_owned());

        // Split the suffix off first: the offset-less form has none, which is
        // exactly what an RFC 3339 parser refuses to accept.
        if let Some(body) = text.strip_suffix('Z') {
            return Ok(ActiveTime::Utc(parse_naive(body).ok_or_else(error)?));
        }

        if text.len() > 6 {
            let (body, suffix) = text.split_at(text.len() - 6);
            if let Some(offset) = parse_offset(suffix) {
                let naive = parse_naive(body).ok_or_else(error)?;
                return Ok(ActiveTime::Offset { naive, offset });
            }
        }

        Ok(ActiveTime::Unspecified(
            parse_naive(text).ok_or_else(error)?,
        ))
    }
}

fn parse_naive(text: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f").ok()
}

/// Parses exactly `±hh:mm`.
fn parse_offset(suffix: &str) -> Option<FixedOffset> {
    let bytes = suffix.as_bytes();
    if bytes.len() != 6 || bytes[3] != b':' {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours: i32 = suffix.get(1..3)?.parse().ok()?;
    let minutes: i32 = suffix.get(4..6)?.parse().ok()?;
    FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))
}

impl fmt::Display for ActiveTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_wire())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognised active-activity timestamp: {:?}", self.0)
    }
}

impl std::error::Error for ParseError {}

impl Serialize for ActiveTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_wire())
    }
}

impl<'de> Deserialize<'de> for ActiveTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        ActiveTime::parse(&text).map_err(|e| D::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::read_json_fixture;

    /// Every value the C# wrote must round-trip byte-identically.
    #[test]
    fn the_recorded_csharp_output_round_trips() {
        let fixture = read_json_fixture("active-activities.json");
        let stored = fixture["afterThreeStarts"]
            .as_str()
            .expect("the recorded payload");
        let entries: serde_json::Value = serde_json::from_str(stored).expect("parses");

        let mut checked = 0;
        for entry in entries.as_array().expect("array") {
            let raw = entry["startTime"].as_str().expect("string");
            let parsed = ActiveTime::parse(raw).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(parsed.to_wire(), raw, "did not round-trip");
            checked += 1;
        }
        assert_eq!(checked, 5, "the fixture records five start times");
    }

    #[test]
    fn all_three_kinds_are_distinguished() {
        assert!(matches!(
            ActiveTime::parse("2026-06-15T10:00:00Z").expect("ok"),
            ActiveTime::Utc(_)
        ));
        assert!(matches!(
            ActiveTime::parse("2026-08-28T15:43:21-07:00").expect("ok"),
            ActiveTime::Offset { .. }
        ));
        // The form TrainerTime's RFC 3339 parser would reject outright.
        assert!(matches!(
            ActiveTime::parse("2026-01-01T00:00:00").expect("ok"),
            ActiveTime::Unspecified(_)
        ));
    }

    #[test]
    fn offsets_use_the_full_form_not_the_hour_only_one() {
        let value = ActiveTime::parse("2026-08-28T15:43:21-07:00").expect("ok");
        assert_eq!(value.to_wire(), "2026-08-28T15:43:21-07:00");
        assert!(!value.to_wire().ends_with("-07"));
    }

    #[test]
    fn fractional_seconds_trim_trailing_zeros() {
        for raw in [
            "2026-08-28T15:43:21.1234567-07:00",
            "2026-08-28T15:43:21.1-07:00",
            "2026-08-28T15:43:21-07:00",
        ] {
            let parsed = ActiveTime::parse(raw).expect("ok");
            assert_eq!(parsed.to_wire(), raw);
        }
    }

    #[test]
    fn sub_tick_precision_is_not_invented() {
        // .NET ticks are 100ns, so nothing finer can appear on the wire.
        let parsed = ActiveTime::parse("2026-08-28T15:43:21.0000001Z").expect("ok");
        assert_eq!(parsed.to_wire(), "2026-08-28T15:43:21.0000001Z");
    }

    #[test]
    fn positive_offsets_are_formatted_with_a_colon() {
        let parsed = ActiveTime::parse("2026-01-01T00:00:00+05:30").expect("ok");
        assert_eq!(parsed.to_wire(), "2026-01-01T00:00:00+05:30");
    }

    #[test]
    fn malformed_input_is_an_error_rather_than_a_panic() {
        for raw in ["", "nope", "2026-13-45T99:99:99", "+05:30", "Z"] {
            assert!(ActiveTime::parse(raw).is_err(), "{raw:?} should fail");
        }
    }

    #[test]
    fn serde_agrees_with_parse_and_to_wire() {
        let json = "\"2026-08-28T15:43:21.1234567-07:00\"";
        let value: ActiveTime = serde_json::from_str(json).expect("deserializes");
        assert_eq!(serde_json::to_string(&value).expect("serializes"), json);
    }
}
