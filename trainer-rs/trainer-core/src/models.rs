//! Domain models, ported from `Trainer/Models/*.cs`.
//!
//! Field order follows the C# record declaration order, not alphabetical order,
//! because serialized output must match the existing bytes exactly.
//!
//! # Two serialization formats
//!
//! The shipping app serializes these types two different ways, and the port has
//! to reproduce both:
//!
//! | | `DefaultIgnoreCondition` | unset optional field |
//! |---|---|---|
//! | `ExportImportService` | `WhenWritingNull` | omitted |
//! | `IndexedDbStorageService` | *(unset)* | `"durationSeconds":null` |
//!
//! Since a serde `derive` cannot switch behavior at runtime, serialization goes
//! through the [`Fmt`] wrapper, which carries the desired [`Format`].
//! Deserialization needs no such split — a missing field and an explicit null
//! both read as `None`.

use crate::datetime::TrainerTime;
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

/// Which of the two serializer configurations to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// `ExportImportService`: unset optional fields are omitted.
    Export,
    /// `IndexedDbStorageService`: unset optional fields are written as `null`.
    Storage,
}

/// Pairs a value with the format it should serialize as.
///
/// ```ignore
/// serde_json::to_string(&Fmt(&activity, Format::Export))
/// ```
pub struct Fmt<'a, T: ?Sized>(pub &'a T, pub Format);

/// Writes an optional field per the active format.
fn optional_field<S, T>(
    state: &mut S,
    name: &'static str,
    value: &Option<T>,
    format: Format,
) -> Result<(), S::Error>
where
    S: SerializeStruct,
    T: Serialize,
{
    match (value, format) {
        (Some(v), _) => state.serialize_field(name, v),
        // Unit serializes as JSON null.
        (None, Format::Storage) => state.serialize_field(name, &()),
        (None, Format::Export) => state.skip_field(name),
    }
}

/// Generates the `Vec` and week-map wrappers for a model type.
///
/// These are written per concrete type rather than as blanket impls over
/// `Vec<T>`: a blanket impl bounded by `for<'b> Fmt<'b, T>: Serialize` sends
/// trait resolution into an infinite `Fmt<Vec<_>>` chain during inference and
/// fails with E0275.
macro_rules! impl_collection_formats {
    ($ty:ty) => {
        impl Serialize for Fmt<'_, Vec<$ty>> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for item in self.0 {
                    seq.serialize_element(&Fmt(item, self.1))?;
                }
                seq.end()
            }
        }

        impl Serialize for Fmt<'_, BTreeMap<String, Vec<$ty>>> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(self.0.len()))?;
                for (key, value) in self.0 {
                    map.serialize_entry(key, &Fmt(value, self.1))?;
                }
                map.end()
            }
        }
    };
}

/// `Trainer/Models/NetBenefit.cs`.
///
/// `System.Text.Json` serializes enums as integers by default and accepts any
/// integer on read, including values outside the declared set. [`Self::Other`]
/// keeps such a value intact rather than silently collapsing it to the default,
/// which would corrupt data on the next write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetBenefit {
    #[default]
    Neutral,
    Positive,
    Negative,
    Other(i32),
}

impl NetBenefit {
    pub fn as_i32(self) -> i32 {
        match self {
            NetBenefit::Neutral => 0,
            NetBenefit::Positive => 1,
            NetBenefit::Negative => 2,
            NetBenefit::Other(v) => v,
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => NetBenefit::Neutral,
            1 => NetBenefit::Positive,
            2 => NetBenefit::Negative,
            other => NetBenefit::Other(other),
        }
    }
}

impl Serialize for NetBenefit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i32(self.as_i32())
    }
}

impl<'de> Deserialize<'de> for NetBenefit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(NetBenefit::from_i32(i32::deserialize(deserializer)?))
    }
}

/// `Trainer/Models/DurationOption.cs`. A UI filter selection, never persisted,
/// so it carries no serde implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationOption {
    Last24Hours,
    Last7Days,
    Week,
    Last4Weeks,
}

/// `Trainer/Models/Activity.cs`.
///
/// `notes` has three distinct states and they must stay distinct: `None`
/// (null in storage, omitted in exports), `Some("")`, and `Some(text)`. In the
/// captured profile these occur 50, 38, and 439 times. Folding `Some("")` into
/// `None` — which is what `EmptyStringAsNullConverter` would have done had it
/// ever been wired up — corrupts 38 activities.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: i32,
    pub activity_type_id: i32,
    pub when: TrainerTime,
    pub amount: i32,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub duration_seconds: Option<i32>,
    #[serde(default)]
    pub known_location_id: Option<i32>,
}

impl Serialize for Fmt<'_, Activity> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let a = self.0;
        let mut state = serializer.serialize_struct("Activity", 7)?;
        state.serialize_field("id", &a.id)?;
        state.serialize_field("activityTypeId", &a.activity_type_id)?;
        state.serialize_field("when", &a.when)?;
        state.serialize_field("amount", &a.amount)?;
        optional_field(&mut state, "notes", &a.notes, self.1)?;
        optional_field(&mut state, "durationSeconds", &a.duration_seconds, self.1)?;
        optional_field(&mut state, "knownLocationId", &a.known_location_id, self.1)?;
        state.end()
    }
}

/// `Trainer/Models/ActivityType.cs`.
///
/// `decimal_places` scales stored amounts: an amount of 125 with
/// `decimal_places` of 2 displays as 1.25. Amounts are therefore always
/// integers on the wire.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityType {
    pub id: i32,
    pub name: String,
    pub net_benefit: NetBenefit,
    #[serde(default)]
    pub daily_amount: Option<i32>,
    #[serde(default)]
    pub weekly_amount: Option<i32>,
    #[serde(default)]
    pub unit: Option<String>,
    pub is_private: bool,
    pub decimal_places: i32,
}

impl Serialize for Fmt<'_, ActivityType> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let t = self.0;
        let mut state = serializer.serialize_struct("ActivityType", 8)?;
        state.serialize_field("id", &t.id)?;
        state.serialize_field("name", &t.name)?;
        state.serialize_field("netBenefit", &t.net_benefit)?;
        optional_field(&mut state, "dailyAmount", &t.daily_amount, self.1)?;
        optional_field(&mut state, "weeklyAmount", &t.weekly_amount, self.1)?;
        optional_field(&mut state, "unit", &t.unit, self.1)?;
        state.serialize_field("isPrivate", &t.is_private)?;
        state.serialize_field("decimalPlaces", &t.decimal_places)?;
        state.end()
    }
}

/// `Trainer/Models/KnownLocation.cs`.
///
/// Ids come from `HashCode.Combine`, which .NET seeds randomly per process, so
/// they are large values of either sign and are not reproducible by any
/// implementation. They are preserved verbatim and never regenerated.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownLocation {
    pub id: i32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl Serialize for Fmt<'_, KnownLocation> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // No optional fields, so both formats agree.
        let l = self.0;
        let mut state = serializer.serialize_struct("KnownLocation", 4)?;
        state.serialize_field("id", &l.id)?;
        state.serialize_field("name", &l.name)?;
        state.serialize_field("latitude", &l.latitude)?;
        state.serialize_field("longitude", &l.longitude)?;
        state.end()
    }
}

impl_collection_formats!(Activity);
impl_collection_formats!(ActivityType);
impl_collection_formats!(KnownLocation);

/// The export file produced by `ExportImportService.ExportDataAsync`.
///
/// Activities arrive grouped by week key. A `BTreeMap` is used rather than
/// preserving insertion order: week keys are zero-padded `YYYY.WW`, so lexical
/// ordering is chronological, and the captured export is already in that order.
/// The C# side builds a `Dictionary` from a `GroupBy`, whose order follows the
/// source list, so a differently ordered source would emit differently ordered
/// keys — semantically identical and still importable, just not byte-identical.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDocument {
    pub activities: BTreeMap<String, Vec<Activity>>,
    pub activity_types: Vec<ActivityType>,
    pub known_locations: Vec<KnownLocation>,
    pub export_date: TrainerTime,
}

impl Serialize for Fmt<'_, ExportDocument> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let d = self.0;
        let mut state = serializer.serialize_struct("ExportDocument", 4)?;
        state.serialize_field("activities", &Fmt(&d.activities, self.1))?;
        state.serialize_field("activityTypes", &Fmt(&d.activity_types, self.1))?;
        state.serialize_field("knownLocations", &Fmt(&d.known_locations, self.1))?;
        state.serialize_field("exportDate", &d.export_date)?;
        state.end()
    }
}

/// Serializes any model in the given format.
pub fn to_json<T>(value: &T, format: Format) -> Result<String, serde_json::Error>
where
    for<'a> Fmt<'a, T>: Serialize,
{
    // Must go through the .NET-compatible escaper, not serde_json::to_string:
    // System.Text.Json escapes `&'+<>` plus everything non-ASCII, so a positive
    // UTC offset alone ("+05:45" -> "\u002B05:45") would break byte-identity.
    crate::escaping::to_string(&Fmt(value, format))
}

#[cfg(test)]
mod tests {
    //! Task 4.3. The `timestamps-export-*` and `timestamps-storage-*` fixtures
    //! are raw `System.Text.Json` output of `List<Activity>` under each of the
    //! two configurations, so they are exact byte-identity targets.

    use super::*;
    use crate::fixtures::{fixture_dir, read_fixture, read_json_fixture};

    fn fixture_names(prefix: &str) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(fixture_dir())
            .expect("fixture directory is readable")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(prefix) && n.ends_with(".json"))
            .collect();
        names.sort();
        names
    }

    #[test]
    fn export_document_round_trips_byte_identically() {
        let raw = read_fixture("export.json");
        let parsed: ExportDocument =
            serde_json::from_str(&raw).expect("export.json parses into typed models");

        assert_eq!(parsed.activities.len(), 31);
        assert_eq!(parsed.activity_types.len(), 16);
        assert_eq!(parsed.known_locations.len(), 11);

        assert_eq!(
            to_json(&parsed, Format::Export).expect("serializes"),
            raw,
            "the de-identified real export must round-trip byte-for-byte"
        );
    }

    #[test]
    fn csharp_export_arrays_round_trip_byte_identically() {
        let names = fixture_names("timestamps-export-");
        assert!(
            names.len() >= 6,
            "expected six timezone fixtures, got {names:?}"
        );

        for name in &names {
            let raw = read_fixture(name);
            let parsed: Vec<Activity> =
                serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                to_json(&parsed, Format::Export).expect("serializes"),
                raw,
                "{name} did not round-trip in Export format"
            );
        }
    }

    #[test]
    fn csharp_storage_arrays_round_trip_byte_identically() {
        let names = fixture_names("timestamps-storage-");
        assert!(
            names.len() >= 6,
            "expected six timezone fixtures, got {names:?}"
        );

        for name in &names {
            let raw = read_fixture(name);
            let parsed: Vec<Activity> =
                serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                to_json(&parsed, Format::Storage).expect("serializes"),
                raw,
                "{name} did not round-trip in Storage format"
            );
        }
    }

    #[test]
    fn the_two_formats_actually_differ() {
        // Guards against both formats accidentally becoming the same thing,
        // which would let every other test here pass while the storage path
        // silently emitted export-shaped JSON.
        let export = read_fixture("timestamps-export-UTC.json");
        let storage = read_fixture("timestamps-storage-UTC.json");
        assert_ne!(export, storage);
        assert!(!export.contains(":null"), "exports omit unset fields");
        assert!(
            storage.contains(":null"),
            "storage writes unset fields as null"
        );
    }

    #[test]
    fn notes_keeps_three_distinct_states() {
        let snapshot = read_json_fixture("idb-snapshot.json");
        let entries = snapshot["entries"]
            .as_object()
            .expect("snapshot has entries");

        let (mut null, mut empty, mut text) = (0, 0, 0);
        for (key, entry) in entries {
            if !key.starts_with("activities-") {
                continue;
            }
            let bucket: Vec<Activity> = serde_json::from_value(entry["value"].clone())
                .unwrap_or_else(|e| panic!("{key}: {e}"));
            for activity in bucket {
                match activity.notes.as_deref() {
                    None => null += 1,
                    Some("") => empty += 1,
                    Some(_) => text += 1,
                }
            }
        }

        // The counts measured from the real profile. If empty strings were
        // folded into None, `empty` would be 0 and `null` would be 88.
        assert_eq!((null, empty, text), (50, 38, 439));
    }

    #[test]
    fn optional_fields_switch_between_omitted_and_null() {
        let activity = Activity {
            id: 1,
            activity_type_id: 2,
            when: crate::datetime::TrainerTime::parse("2026-01-01T08:56:44-08").expect("parses"),
            amount: 15,
            notes: None,
            duration_seconds: None,
            known_location_id: None,
        };

        assert_eq!(
            to_json(&activity, Format::Export).expect("serializes"),
            r#"{"id":1,"activityTypeId":2,"when":"2026-01-01T08:56:44-08","amount":15}"#
        );
        assert_eq!(
            to_json(&activity, Format::Storage).expect("serializes"),
            r#"{"id":1,"activityTypeId":2,"when":"2026-01-01T08:56:44-08","amount":15,"notes":null,"durationSeconds":null,"knownLocationId":null}"#
        );
    }

    #[test]
    fn empty_notes_are_written_not_omitted() {
        let mut activity = Activity {
            id: 1,
            activity_type_id: 2,
            when: crate::datetime::TrainerTime::parse("2026-06-15T10:00:00Z").expect("parses"),
            amount: 1,
            notes: Some(String::new()),
            duration_seconds: None,
            known_location_id: None,
        };

        for format in [Format::Export, Format::Storage] {
            assert!(
                to_json(&activity, format)
                    .expect("serializes")
                    .contains(r#""notes":"""#),
                "empty notes must serialize as an empty string in {format:?}"
            );
        }

        activity.notes = None;
        assert!(
            !to_json(&activity, Format::Export)
                .expect("serializes")
                .contains("notes")
        );
    }

    #[test]
    fn net_benefit_serializes_as_an_integer_and_keeps_unknown_values() {
        assert_eq!(NetBenefit::Neutral.as_i32(), 0);
        assert_eq!(NetBenefit::Positive.as_i32(), 1);
        assert_eq!(NetBenefit::Negative.as_i32(), 2);

        // System.Text.Json accepts undefined enum integers, so a value outside
        // the declared set must survive a read/write cycle rather than being
        // collapsed to the default.
        let odd: NetBenefit = serde_json::from_str("7").expect("deserializes");
        assert_eq!(odd, NetBenefit::Other(7));
        assert_eq!(serde_json::to_string(&odd).expect("serializes"), "7");
    }

    #[test]
    fn activity_type_variety_from_the_real_profile_round_trips() {
        let raw = read_fixture("export.json");
        let parsed: ExportDocument = serde_json::from_str(&raw).expect("parses");

        // The captured profile exercises all three net benefits, both privacy
        // flags, and both decimal-place settings.
        let benefits: std::collections::BTreeSet<i32> = parsed
            .activity_types
            .iter()
            .map(|t| t.net_benefit.as_i32())
            .collect();
        assert_eq!(benefits, [0, 1, 2].into_iter().collect());
        assert!(parsed.activity_types.iter().any(|t| t.is_private));
        assert!(parsed.activity_types.iter().any(|t| !t.is_private));
        assert!(parsed.activity_types.iter().any(|t| t.decimal_places == 2));
    }
}
