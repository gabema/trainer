//! Timestamp serialization matching the C# `DateTimeConverter` byte-for-byte.
//!
//! The stored format is *not* RFC 3339. When a UTC offset has a zero minute
//! component the converter emits it hour-only — `-08`, not `-08:00` — which
//! `chrono`'s RFC 3339 helpers can neither produce nor parse. .NET cannot parse
//! its own output either, which is why the C# `Read` path carries a regex that
//! rewrites `-08` to `-08:00` before handing the string to `DateTimeOffset.Parse`.
//!
//! `chrono` is therefore used for calendar arithmetic only; the wire format is
//! handled here.

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// `yyyy-MM-ddTHH:mm:ss` — seconds precision, no fractional part.
const WIRE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

/// A timestamp as the app actually stores one.
///
/// .NET pairs a naive `DateTime` with a `DateTimeKind` of `Utc`, `Local`, or
/// `Unspecified`. On write the converter branches only two ways: `Utc` emits a
/// trailing `Z`, and both `Local` and `Unspecified` resolve an offset from
/// `TimeZoneInfo.Local` and emit that. So only two states are observable on the
/// wire, and this type mirrors those rather than .NET's three kinds.
///
/// See `docs` on [`TrainerTime::parse`] for why the offset is retained here when
/// the C# implementation discards it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainerTime {
    /// `DateTimeKind.Utc`. Serializes with a trailing `Z`.
    Utc(NaiveDateTime),
    /// `DateTimeKind.Local` or `Unspecified`, carrying its resolved offset.
    Offset {
        /// Wall-clock time as displayed and as bucketed by `WeekHelper`.
        naive: NaiveDateTime,
        offset: FixedOffset,
    },
}

impl TrainerTime {
    /// The wall-clock time. This is what the UI displays and what week
    /// bucketing keys off, so it is identical under either variant.
    pub fn naive(&self) -> NaiveDateTime {
        match self {
            TrainerTime::Utc(n) => *n,
            TrainerTime::Offset { naive, .. } => *naive,
        }
    }

    /// `default(DateTime)` in .NET — `DateTime.MinValue`, kind `Unspecified`.
    /// Reached only from a null or empty `when`, which the serializer never
    /// emits because `Activity.When` is non-nullable.
    fn dotnet_default() -> Self {
        let naive = NaiveDate::from_ymd_opt(1, 1, 1)
            .expect("0001-01-01 is a valid date")
            .and_hms_opt(0, 0, 0)
            .expect("midnight is a valid time");
        TrainerTime::Utc(naive)
    }

    /// Renders the wire format, reproducing `DateTimeConverter.Write`.
    pub fn to_wire(&self) -> String {
        match self {
            TrainerTime::Utc(naive) => format!("{}Z", naive.format(WIRE_FORMAT)),
            TrainerTime::Offset { naive, offset } => {
                format!("{}{}", naive.format(WIRE_FORMAT), format_offset(*offset))
            }
        }
    }

    /// Parses the wire format, reproducing `DateTimeConverter.Read`.
    ///
    /// **Deliberate divergence.** The C# reader returns `dto.DateTime` for a
    /// non-zero offset, which yields the wall clock with kind `Unspecified` and
    /// *discards the parsed offset*. A later write then recomputes the offset
    /// from whatever `TimeZoneInfo.Local` says at that moment, so data written
    /// in one timezone and re-saved in another keeps its wall clock but silently
    /// moves to a different instant — verified against the C# implementation and
    /// recorded in `timestamps-crosszone-*` fixtures.
    ///
    /// This implementation retains the offset instead. Every value round-trips
    /// byte-identically regardless of the machine's timezone, nothing that is
    /// persisted changes for a stationary user, and week bucketing is unaffected
    /// because it keys off the wall clock, which both behaviors preserve.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        if s.is_empty() {
            return Ok(Self::dotnet_default());
        }

        let normalized = normalize_hour_only_offset(s);
        let parsed =
            DateTime::parse_from_rfc3339(&normalized).map_err(|_| ParseError(s.to_owned()))?;

        if parsed.offset().local_minus_utc() == 0 {
            Ok(TrainerTime::Utc(parsed.naive_local()))
        } else {
            Ok(TrainerTime::Offset {
                naive: parsed.naive_local(),
                offset: *parsed.offset(),
            })
        }
    }
}

impl fmt::Display for TrainerTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_wire())
    }
}

/// A timestamp string that could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognised timestamp: {:?}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// Reproduces `DateTimeConverter.FormatOffset`.
///
/// ```text
/// zero            -> "Z"
/// whole hours     -> "-08"      (hour-only; NOT RFC 3339)
/// with minutes    -> "+05:30", "-03:30"
/// ```
///
/// The integer arithmetic mirrors the C# exactly: `hours` truncates toward zero
/// in both languages, `minutes` takes the absolute value of a remainder that
/// carries the sign of the dividend, and the sign is decided from the total
/// rather than from `hours` — which matters for offsets between -1 and 0 hours,
/// where `hours` is 0 and carries no sign of its own.
fn format_offset(offset: FixedOffset) -> String {
    let total_minutes = offset.local_minus_utc() / 60;
    if total_minutes == 0 {
        return "Z".to_owned();
    }

    let hours = total_minutes / 60;
    let minutes = (total_minutes % 60).abs();
    let sign = if total_minutes >= 0 { '+' } else { '-' };
    let h = hours.abs();

    if minutes == 0 {
        format!("{sign}{h:02}")
    } else {
        format!("{sign}{h:02}:{minutes:02}")
    }
}

/// Rewrites a trailing hour-only offset so an RFC 3339 parser accepts it:
/// `…-08` becomes `…-08:00`. Equivalent to the C# regex `([+-])(\d{1,2})$`.
///
/// Must not fire on `…+05:30` (the trailing digits are preceded by `:`, not a
/// sign) or on `…Z`.
fn normalize_hour_only_offset(s: &str) -> String {
    let bytes = s.as_bytes();
    let digits = bytes
        .iter()
        .rev()
        .take_while(|b| b.is_ascii_digit())
        .count();

    // The length guard must precede the subtraction: for an all-digit input such
    // as "12", `len - digits - 1` underflows and panics.
    if (1..=2).contains(&digits) && bytes.len() > digits {
        let sign_index = bytes.len() - digits - 1;
        if bytes[sign_index] == b'+' || bytes[sign_index] == b'-' {
            return format!("{s}:00");
        }
    }

    s.to_owned()
}

impl Serialize for TrainerTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_wire())
    }
}

impl<'de> Deserialize<'de> for TrainerTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // The C# reader maps both a null token and an empty string to
        // default(DateTime) rather than failing.
        let raw = Option::<String>::deserialize(deserializer)?;
        match raw {
            None => Ok(Self::dotnet_default()),
            Some(s) => TrainerTime::parse(&s)
                .map_err(|_| D::Error::invalid_value(Unexpected::Str(&s), &"a Trainer timestamp")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset(hours: i32, minutes: i32) -> FixedOffset {
        let sign = if hours < 0 || minutes < 0 { -1 } else { 1 };
        FixedOffset::east_opt(sign * (hours.abs() * 3600 + minutes.abs() * 60))
            .expect("valid offset")
    }

    #[test]
    fn format_offset_matches_dotnet() {
        assert_eq!(format_offset(offset(0, 0)), "Z");
        assert_eq!(format_offset(offset(-8, 0)), "-08");
        assert_eq!(format_offset(offset(-7, 0)), "-07");
        assert_eq!(format_offset(offset(5, 30)), "+05:30");
        assert_eq!(format_offset(offset(5, 45)), "+05:45");
        assert_eq!(format_offset(offset(8, 45)), "+08:45");
        assert_eq!(format_offset(offset(-3, 30)), "-03:30");
        assert_eq!(format_offset(offset(-2, 30)), "-02:30");
    }

    #[test]
    fn format_offset_handles_sub_hour_negatives() {
        // Between -1h and 0, `hours` is 0 and carries no sign, so the sign must
        // come from the total. Getting this wrong yields "+00:30" for -00:30.
        assert_eq!(format_offset(offset(0, -30)), "-00:30");
        assert_eq!(format_offset(offset(0, 30)), "+00:30");
    }

    #[test]
    fn normalize_only_touches_hour_only_offsets() {
        assert_eq!(
            normalize_hour_only_offset("2026-01-01T08:56:44-08"),
            "2026-01-01T08:56:44-08:00"
        );
        assert_eq!(
            normalize_hour_only_offset("2026-01-01T08:56:44-8"),
            "2026-01-01T08:56:44-8:00"
        );
        // Already has minutes — must be left alone.
        assert_eq!(
            normalize_hour_only_offset("2026-01-01T08:56:44+05:30"),
            "2026-01-01T08:56:44+05:30"
        );
        // Zero offset marker — must be left alone.
        assert_eq!(
            normalize_hour_only_offset("2026-06-15T10:00:00Z"),
            "2026-06-15T10:00:00Z"
        );
    }

    #[test]
    fn utc_kind_round_trips() {
        let t = TrainerTime::parse("2026-06-15T10:00:00Z").expect("parses");
        assert!(matches!(t, TrainerTime::Utc(_)));
        assert_eq!(t.to_wire(), "2026-06-15T10:00:00Z");
    }

    #[test]
    fn hour_only_offset_round_trips() {
        let t = TrainerTime::parse("2026-01-01T08:56:44-08").expect("parses");
        assert_eq!(t.to_wire(), "2026-01-01T08:56:44-08");
        // The wall clock is preserved, which is what display and week bucketing use.
        assert_eq!(t.naive().to_string(), "2026-01-01 08:56:44");
    }

    #[test]
    fn empty_string_yields_dotnet_default() {
        assert_eq!(
            TrainerTime::parse("").expect("parses"),
            TrainerTime::dotnet_default()
        );
    }

    #[test]
    fn unparseable_input_is_an_error() {
        assert!(TrainerTime::parse("not a timestamp").is_err());
    }

    #[test]
    fn malformed_input_does_not_panic() {
        // Regression: normalize_hour_only_offset subtracted before checking the
        // length, so an all-digit string underflowed and panicked rather than
        // returning an error. Reachable from any corrupt stored value.
        for raw in ["12", "8", "1", "+", "-", "+8", "-08", "", "T::", "99999"] {
            let _ = TrainerTime::parse(raw);
        }
    }

    #[test]
    fn bare_sign_and_hour_is_normalized_like_the_csharp_regex() {
        // The C# regex ([+-])(\d{1,2})$ matches a single-digit hour too.
        assert_eq!(normalize_hour_only_offset("+8"), "+8:00");
        assert_eq!(normalize_hour_only_offset("12"), "12");
        assert_eq!(normalize_hour_only_offset("99999"), "99999");
    }
}

#[cfg(test)]
mod fixture_round_trip {
    //! Task 3.4 — every timestamp the C# implementation produced must survive
    //! parse-then-serialize byte-identically.
    //!
    //! These run natively rather than needing `TZ` set, because parsing retains
    //! the offset instead of re-deriving it from the ambient timezone.

    use super::*;
    use crate::fixtures::{fixture_dir, read_json_fixture};
    use std::collections::BTreeSet;

    /// Pulls every `when` value out of a fixture, whatever its shape:
    /// a flat array of activities, or the export's week-keyed buckets.
    fn collect_timestamps(value: &serde_json::Value) -> Vec<String> {
        fn walk(v: &serde_json::Value, out: &mut Vec<String>) {
            match v {
                serde_json::Value::Object(map) => {
                    for (key, val) in map {
                        if key == "when" {
                            if let Some(s) = val.as_str() {
                                out.push(s.to_owned());
                            }
                        } else {
                            walk(val, out);
                        }
                    }
                }
                serde_json::Value::Array(items) => {
                    for item in items {
                        walk(item, out);
                    }
                }
                _ => {}
            }
        }

        let mut out = Vec::new();
        walk(value, &mut out);
        out
    }

    fn timestamp_fixture_names() -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(fixture_dir())
            .expect("fixture directory is readable")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with("timestamps-") && name.ends_with(".json"))
            .collect();
        names.sort();
        names
    }

    #[test]
    fn every_timestamp_fixture_round_trips_byte_identically() {
        let names = timestamp_fixture_names();
        assert!(
            names.len() >= 18,
            "expected the committed timestamp fixtures across six timezones, found {}: {names:?}",
            names.len()
        );

        let mut checked = 0usize;
        let mut offset_forms = BTreeSet::new();

        for name in &names {
            let value = read_json_fixture(name);
            let timestamps = collect_timestamps(&value);
            assert!(!timestamps.is_empty(), "{name} contained no timestamps");

            for raw in timestamps {
                let parsed = TrainerTime::parse(&raw).unwrap_or_else(|e| panic!("{name}: {e}"));
                assert_eq!(
                    parsed.to_wire(),
                    raw,
                    "{name}: timestamp did not round-trip"
                );

                offset_forms.insert(raw[19..].to_owned());
                checked += 1;
            }
        }

        // Guard against the suite silently passing because it read nothing, and
        // prove every branch of FormatOffset was exercised by real C# output.
        assert!(checked >= 180, "only {checked} timestamps checked");
        for expected in [
            "Z", "-08", "-07", "+05:30", "+05:45", "+08:45", "-03:30", "-02:30",
        ] {
            assert!(
                offset_forms.contains(expected),
                "no fixture exercised the {expected:?} offset form; saw {offset_forms:?}"
            );
        }
    }

    #[test]
    fn every_real_export_timestamp_round_trips() {
        let value = read_json_fixture("export.json");
        let timestamps = collect_timestamps(&value);
        assert_eq!(
            timestamps.len(),
            527,
            "the de-identified export has 527 activities"
        );

        for raw in timestamps {
            let parsed = TrainerTime::parse(&raw).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(parsed.to_wire(), raw);
        }
    }

    #[test]
    fn serde_round_trips_through_json() {
        // The Serialize/Deserialize impls must agree with parse/to_wire, since
        // section 4 reaches them through derived model structs rather than directly.
        for raw in [
            "2026-01-01T08:56:44-08",
            "2026-06-15T10:00:00Z",
            "2026-01-01T00:00:00+05:45",
            "2026-01-01T00:00:00-03:30",
        ] {
            let json = format!("\"{raw}\"");
            let value: TrainerTime = serde_json::from_str(&json).expect("deserializes");
            assert_eq!(serde_json::to_string(&value).expect("serializes"), json);
        }
    }

    #[test]
    fn null_timestamp_deserializes_to_dotnet_default() {
        let value: TrainerTime = serde_json::from_str("null").expect("null is tolerated");
        assert_eq!(value.naive().to_string(), "0001-01-01 00:00:00");
    }
}
