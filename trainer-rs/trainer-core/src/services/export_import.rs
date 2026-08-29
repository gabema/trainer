//! Export and import, ported from `Trainer/Services/ExportImportService.cs`.
//!
//! Exports use the **export** serializer configuration (unset optional fields
//! omitted), which is why this is the one place the two configurations must not
//! be confused.

use super::activity::ActivityService;
use super::known_location::KnownLocationService;
use crate::datetime::TrainerTime;
use crate::models::{self, Activity, ActivityType, ExportDocument, Format, KnownLocation};
use crate::storage::buckets::WeekBucketed;
use crate::storage::{Storage, StorageError, StorageResult};
use crate::week;
use chrono::NaiveDateTime;
use std::collections::BTreeMap;

/// An import that could not be applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError(pub String);

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid import data format: {}", self.0)
    }
}

impl std::error::Error for ImportError {}

impl From<StorageError> for ImportError {
    fn from(value: StorageError) -> Self {
        ImportError(value.to_string())
    }
}

pub struct ExportImportService<'a, S: Storage> {
    storage: &'a WeekBucketed<S>,
}

impl<'a, S: Storage> ExportImportService<'a, S> {
    pub fn new(storage: &'a WeekBucketed<S>) -> Self {
        Self { storage }
    }

    /// Serializes everything, grouped by week, with `exported_at` as the stamp.
    ///
    /// The C# uses `DateTime.UtcNow`; the caller supplies it here so the output
    /// is deterministic and testable.
    pub async fn export(&self, exported_at: NaiveDateTime) -> StorageResult<String> {
        let activity_types: Vec<ActivityType> = match self.storage.get_item("activityTypes").await?
        {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| StorageError::new("deserialize", "activityTypes", e.to_string()))?,
            None => Vec::new(),
        };

        let all_activities: Vec<Activity> = {
            let json = self
                .storage
                .get_item("activities")
                .await?
                .unwrap_or_else(|| "[]".to_owned());
            serde_json::from_str(&json)
                .map_err(|e| StorageError::new("deserialize", "activities", e.to_string()))?
        };

        let mut activities: BTreeMap<String, Vec<Activity>> = BTreeMap::new();
        for activity in all_activities {
            activities
                .entry(week::week_key(activity.when.naive()))
                .or_default()
                .push(activity);
        }

        let known_locations = KnownLocationService::new(self.storage.inner())
            .all()
            .await?;

        let document = ExportDocument {
            activities,
            activity_types,
            known_locations,
            export_date: TrainerTime::Utc(exported_at),
        };

        models::to_json(&document, Format::Export)
            .map_err(|e| StorageError::new("serialize", "export", e.to_string()))
    }

    /// Replaces all stored data with the contents of an export.
    ///
    /// Accepts both the current week-keyed object and the older flat array, and
    /// both camelCase and PascalCase property names, as the C# does for
    /// backward compatibility.
    pub async fn import(&self, json: &str) -> Result<(), ImportError> {
        let root: serde_json::Value =
            serde_json::from_str(json).map_err(|e| ImportError(e.to_string()))?;
        let object = root
            .as_object()
            .ok_or_else(|| ImportError("the root of an export must be an object".to_owned()))?;

        let pick = |camel: &str, pascal: &str| -> Option<&serde_json::Value> {
            object.get(camel).or_else(|| object.get(pascal))
        };

        // Everything is validated before anything is cleared, so a malformed
        // file cannot leave the profile empty.
        let activities = match pick("activities", "Activities") {
            Some(value) => Some(Self::read_activities(value)?),
            None => None,
        };
        let activity_types: Option<Vec<ActivityType>> = match pick("activityTypes", "ActivityTypes")
        {
            Some(value) => Some(
                serde_json::from_value(value.clone()).map_err(|e| ImportError(e.to_string()))?,
            ),
            None => None,
        };
        let known_locations: Option<Vec<KnownLocation>> =
            match pick("knownLocations", "KnownLocations") {
                Some(value) if value.is_array() => Some(
                    serde_json::from_value(value.clone())
                        .map_err(|e| ImportError(e.to_string()))?,
                ),
                _ => None,
            };

        self.storage.clear().await?;

        let imported_activities = activities.is_some();
        if let Some(by_week) = activities {
            for (week_key, bucket) in by_week {
                self.storage
                    .set_activities_for_week(&week_key, &bucket)
                    .await?;
            }
        }

        if let Some(types) = activity_types {
            let json =
                models::to_json(&types, Format::Storage).map_err(|e| ImportError(e.to_string()))?;
            self.storage.set_item("activityTypes", &json).await?;
        }

        if let Some(locations) = known_locations {
            let service = KnownLocationService::new(self.storage.inner());
            for location in locations {
                service.save(location).await?;
            }
        }

        // Restored ids can be far above the counter, so it must be rebuilt or
        // the next add would collide.
        if imported_activities {
            ActivityService::new(self.storage)
                .recalculate_next_id()
                .await?;
        }

        Ok(())
    }

    /// Reads either shape of the `activities` value, regrouping by week key so
    /// the caller always gets buckets.
    fn read_activities(
        value: &serde_json::Value,
    ) -> Result<BTreeMap<String, Vec<Activity>>, ImportError> {
        let mut by_week: BTreeMap<String, Vec<Activity>> = BTreeMap::new();

        if value.is_array() {
            // Legacy flat array.
            let flat: Vec<Activity> =
                serde_json::from_value(value.clone()).map_err(|e| ImportError(e.to_string()))?;
            for activity in flat {
                by_week
                    .entry(week::week_key(activity.when.naive()))
                    .or_default()
                    .push(activity);
            }
            return Ok(by_week);
        }

        if value.is_object() {
            let keyed: BTreeMap<String, Vec<Activity>> =
                serde_json::from_value(value.clone()).map_err(|e| ImportError(e.to_string()))?;
            return Ok(keyed);
        }

        Err(ImportError(
            "activities must be an array or an object".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{block_on, read_fixture};
    use crate::storage::MemStorage;

    fn now() -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(2026, 8, 28)
            .expect("valid")
            .and_hms_opt(22, 43, 21)
            .expect("valid")
    }

    fn store() -> WeekBucketed<MemStorage> {
        WeekBucketed::new(MemStorage::new())
    }

    /// Ports `ExportDataAsync_HandlesEmptyData`.
    #[test]
    fn exporting_an_empty_profile_produces_empty_collections() {
        block_on(async {
            let storage = store();
            let json = ExportImportService::new(&storage)
                .export(now())
                .await
                .expect("ok");
            assert_eq!(
                json,
                r#"{"activities":{},"activityTypes":[],"knownLocations":[],"exportDate":"2026-08-28T22:43:21Z"}"#
            );
        });
    }

    /// The strongest available check: a real export must survive
    /// import-then-export unchanged. Ports `ExportDataAsync_MinifiesOutput_OmitsNulls_AndRoundTrips`.
    #[test]
    fn a_real_export_round_trips_byte_identically() {
        block_on(async {
            let original = read_fixture("export.json");
            let storage = store();
            let service = ExportImportService::new(&storage);

            service.import(&original).await.expect("import succeeds");

            let document: ExportDocument = serde_json::from_str(&original).expect("fixture parses");
            let exported = service
                .export(document.export_date.naive())
                .await
                .expect("export succeeds");

            assert_eq!(exported, original, "import then export must be lossless");
        });
    }

    /// Ports `ImportDataAsync_ImportsActivitiesAndTypes_NewFormat`.
    #[test]
    fn imports_the_week_keyed_format() {
        block_on(async {
            let storage = store();
            let service = ExportImportService::new(&storage);
            service
                .import(&read_fixture("export.json"))
                .await
                .expect("ok");

            let weeks = storage.available_week_keys().await.expect("ok");
            assert_eq!(weeks.len(), 31);
            assert!(storage.inner().contains("activityTypes"));
            assert!(storage.inner().contains("knownLocations"));
        });
    }

    /// Ports `ImportDataAsync_ImportsActivitiesAndTypes_OldFormat`.
    #[test]
    fn imports_the_legacy_flat_array_by_regrouping_it() {
        block_on(async {
            let json = r#"{"activities":[
                {"id":1,"activityTypeId":1,"when":"2025-12-30T08:00:00-08","amount":1},
                {"id":2,"activityTypeId":1,"when":"2026-01-02T08:00:00-08","amount":2}
            ],"activityTypes":[]}"#;

            let storage = store();
            ExportImportService::new(&storage)
                .import(json)
                .await
                .expect("ok");

            let mut weeks = storage.available_week_keys().await.expect("ok");
            weeks.sort();
            assert_eq!(
                weeks,
                vec!["2025.53", "2026.01"],
                "regrouped across the year boundary"
            );
        });
    }

    /// Ports the PascalCase backward-compatibility branches.
    #[test]
    fn accepts_pascal_case_property_names() {
        block_on(async {
            let json = r#"{"Activities":[
                {"id":1,"activityTypeId":1,"when":"2026-01-02T08:00:00-08","amount":2}
            ],"ActivityTypes":[],"KnownLocations":[]}"#;

            let storage = store();
            ExportImportService::new(&storage)
                .import(json)
                .await
                .expect("ok");
            assert_eq!(storage.available_week_keys().await.expect("ok").len(), 1);
        });
    }

    /// Ports `ImportDataAsync_ThrowsException_WhenInvalidJson`.
    #[test]
    fn invalid_json_is_rejected() {
        block_on(async {
            let storage = store();
            let error = ExportImportService::new(&storage)
                .import("{not json")
                .await
                .expect_err("should fail");
            assert!(error.to_string().starts_with("Invalid import data format"));
        });
    }

    #[test]
    fn a_malformed_import_leaves_existing_data_intact() {
        block_on(async {
            let storage = store();
            let service = ExportImportService::new(&storage);
            service
                .import(&read_fixture("export.json"))
                .await
                .expect("ok");
            let before = storage.inner().snapshot();

            // Structurally valid JSON whose activities cannot be parsed.
            let error = service
                .import(r#"{"activities":[{"id":"not-a-number"}]}"#)
                .await
                .expect_err("should fail");
            assert!(error.to_string().starts_with("Invalid import data format"));

            assert_eq!(
                storage.inner().snapshot(),
                before,
                "validation happens before the store is cleared"
            );
        });
    }

    /// Ports `ImportDataAsync_HandlesPartialData`.
    #[test]
    fn a_partial_export_imports_what_it_has() {
        block_on(async {
            let storage = store();
            ExportImportService::new(&storage)
                .import(r#"{"activityTypes":[{"id":1,"name":"Water","netBenefit":1,"isPrivate":false,"decimalPlaces":0}]}"#)
                .await
                .expect("ok");

            assert!(storage.inner().contains("activityTypes"));
            assert!(storage.available_week_keys().await.expect("ok").is_empty());
        });
    }

    /// Ports `ImportDataAsync_ClearsAllStorageBeforeWritingData`.
    #[test]
    fn importing_clears_previous_data() {
        block_on(async {
            let storage = store();
            storage.inner().set_item("stale", "1").await.expect("ok");

            ExportImportService::new(&storage)
                .import(r#"{"activityTypes":[]}"#)
                .await
                .expect("ok");

            assert!(!storage.inner().contains("stale"));
        });
    }

    /// Ports `ImportDataAsync_RecalculatesNextId_AfterImportingActivities`
    /// and `_DoesNotRecalculateNextId_WhenNoActivitiesImported`.
    #[test]
    fn the_id_counter_is_rebuilt_only_when_activities_were_imported() {
        block_on(async {
            let storage = store();
            let service = ExportImportService::new(&storage);

            service
                .import(r#"{"activities":[{"id":500,"activityTypeId":1,"when":"2026-01-02T08:00:00-08","amount":1}]}"#)
                .await
                .expect("ok");
            assert_eq!(
                storage
                    .inner()
                    .snapshot()
                    .get("activityNextId")
                    .map(String::as_str),
                Some("501")
            );

            let other = store();
            ExportImportService::new(&other)
                .import(r#"{"activityTypes":[]}"#)
                .await
                .expect("ok");
            assert!(
                !other.inner().contains("activityNextId"),
                "no activities, so the counter is left alone"
            );
        });
    }

    /// Ports `ImportDataAsync_ParsesHourOnlyTimezoneOffset`.
    #[test]
    fn hour_only_offsets_are_accepted_on_import() {
        block_on(async {
            let storage = store();
            ExportImportService::new(&storage)
                .import(r#"{"activities":[{"id":1,"activityTypeId":1,"when":"2026-01-02T08:00:00-08","amount":1}]}"#)
                .await
                .expect("hour-only offsets must parse");

            let stored = storage.activities_for_week("2026.01").await.expect("ok");
            assert_eq!(stored[0].when.to_wire(), "2026-01-02T08:00:00-08");
        });
    }

    /// Ports `ImportDataAsync_ActivityRecordsWithLegacyCoordinates_SucceedsAndIgnoresCoordinates`.
    #[test]
    fn legacy_coordinate_fields_on_activities_are_ignored() {
        block_on(async {
            let storage = store();
            ExportImportService::new(&storage)
                .import(r#"{"activities":[{"id":1,"activityTypeId":1,"when":"2026-01-02T08:00:00-08","amount":1,"latitude":37.4,"longitude":-122.1}]}"#)
                .await
                .expect("unknown fields must be tolerated");

            let stored = storage.activities_for_week("2026.01").await.expect("ok");
            assert_eq!(stored.len(), 1);
            // Ports `ExportDataAsync_ActivityRecords_OmitLatitudeAndLongitudeFields`.
            let exported = ExportImportService::new(&storage)
                .export(now())
                .await
                .expect("ok");
            assert!(!exported.contains("latitude") || exported.contains("knownLocations"));
            assert!(!exported.contains(r#""latitude":37.4"#));
        });
    }

    /// Ports `ImportDataAsync_DeserializesNullOrMissingNotesAsNull`.
    #[test]
    fn null_and_missing_notes_both_import_as_absent() {
        block_on(async {
            let storage = store();
            ExportImportService::new(&storage)
                .import(r#"{"activities":[
                    {"id":1,"activityTypeId":1,"when":"2026-01-02T08:00:00-08","amount":1,"notes":null},
                    {"id":2,"activityTypeId":1,"when":"2026-01-02T09:00:00-08","amount":1}
                ]}"#)
                .await
                .expect("ok");

            let stored = storage.activities_for_week("2026.01").await.expect("ok");
            assert!(stored.iter().all(|a| a.notes.is_none()));
        });
    }
}
