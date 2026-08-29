//! The one-time localStorage to IndexedDB migration, ported from
//! `IndexedDbStorageService.MigrateFromLocalStorageAsync`.
//!
//! Profiles whose last use predates the IndexedDB switch still hold a flat
//! `activities` list and an `activityTypes` list in localStorage. On first load
//! these are moved across: activities are split into week buckets, activity
//! types are written unbucketed, and the legacy keys are deleted only after a
//! successful write.
//!
//! **Migration must never prevent startup.** The C# wraps the whole thing in a
//! catch-all that logs and continues, so this returns a [`MigrationReport`]
//! describing what happened rather than an error.

use super::buckets::WeekBucketed;
use super::{Storage, StorageResult};
use crate::models::{self, Activity, ActivityType, Format};
use crate::week;
use std::collections::BTreeMap;

/// Legacy localStorage keys, in the order the C# processes them.
const LEGACY_ACTIVITIES: &str = "activities";
const LEGACY_ACTIVITY_TYPES: &str = "activityTypes";

/// What a migration attempt did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// Week bucket keys written, in sorted order.
    pub buckets_written: Vec<String>,
    /// Number of activities moved across.
    pub activities_migrated: usize,
    /// Number of activity types moved across.
    pub activity_types_migrated: usize,
    /// Legacy keys deleted afterwards.
    pub legacy_keys_removed: Vec<String>,
    /// Non-fatal problems. Their presence never stops startup.
    pub failures: Vec<String>,
}

impl MigrationReport {
    /// Whether anything was actually moved.
    pub fn migrated_anything(&self) -> bool {
        self.activities_migrated > 0 || self.activity_types_migrated > 0
    }
}

/// Moves legacy data from `legacy` into `target`, reporting rather than failing.
///
/// `target` is the bucketing layer, so activities land in `activities-{weekKey}`
/// and everything else passes through unchanged.
pub async fn migrate_from_legacy<L, T>(legacy: &L, target: &WeekBucketed<T>) -> MigrationReport
where
    L: Storage,
    T: Storage,
{
    let mut report = MigrationReport::default();

    match migrate_activities(legacy, target).await {
        Ok(Some((buckets, count))) => {
            report.buckets_written = buckets;
            report.activities_migrated = count;
            match legacy.remove_item(LEGACY_ACTIVITIES).await {
                Ok(()) => report
                    .legacy_keys_removed
                    .push(LEGACY_ACTIVITIES.to_owned()),
                Err(e) => report.failures.push(e.to_string()),
            }
        }
        Ok(None) => {}
        Err(e) => report.failures.push(e.to_string()),
    }

    match migrate_activity_types(legacy, target).await {
        Ok(Some(count)) => {
            report.activity_types_migrated = count;
            match legacy.remove_item(LEGACY_ACTIVITY_TYPES).await {
                Ok(()) => report
                    .legacy_keys_removed
                    .push(LEGACY_ACTIVITY_TYPES.to_owned()),
                Err(e) => report.failures.push(e.to_string()),
            }
        }
        Ok(None) => {}
        Err(e) => report.failures.push(e.to_string()),
    }

    report
}

/// Returns the buckets written and how many activities moved, or `None` when
/// there was nothing to migrate.
async fn migrate_activities<L, T>(
    legacy: &L,
    target: &WeekBucketed<T>,
) -> StorageResult<Option<(Vec<String>, usize)>>
where
    L: Storage,
    T: Storage,
{
    let Some(json) = legacy.get_item(LEGACY_ACTIVITIES).await? else {
        return Ok(None);
    };
    if json.is_empty() {
        return Ok(None);
    }

    let activities: Vec<Activity> = match serde_json::from_str(&json) {
        Ok(parsed) => parsed,
        // The C# lets a JsonException escape into the catch-all, which logs and
        // continues without removing the legacy key, leaving the data in place.
        Err(e) => {
            return Err(super::StorageError::new(
                "deserialize",
                LEGACY_ACTIVITIES,
                e.to_string(),
            ));
        }
    };
    if activities.is_empty() {
        return Ok(None);
    }

    let migrated = activities.len();
    let mut by_week: BTreeMap<String, Vec<Activity>> = BTreeMap::new();
    for activity in activities {
        by_week
            .entry(week::week_key(activity.when.naive()))
            .or_default()
            .push(activity);
    }

    let mut buckets = Vec::new();
    for (week_key, bucket) in &by_week {
        let storage_key = week::storage_key(week_key);
        let serialized = models::to_json(bucket, Format::Storage).map_err(|e| {
            super::StorageError::new("serialize", storage_key.clone(), e.to_string())
        })?;
        target.inner().set_item(&storage_key, &serialized).await?;
        buckets.push(storage_key);
    }

    Ok(Some((buckets, migrated)))
}

async fn migrate_activity_types<L, T>(
    legacy: &L,
    target: &WeekBucketed<T>,
) -> StorageResult<Option<usize>>
where
    L: Storage,
    T: Storage,
{
    let Some(json) = legacy.get_item(LEGACY_ACTIVITY_TYPES).await? else {
        return Ok(None);
    };
    if json.is_empty() {
        return Ok(None);
    }

    let types: Vec<ActivityType> = serde_json::from_str(&json).map_err(|e| {
        super::StorageError::new("deserialize", LEGACY_ACTIVITY_TYPES, e.to_string())
    })?;
    if types.is_empty() {
        return Ok(None);
    }

    let serialized = models::to_json(&types, Format::Storage)
        .map_err(|e| super::StorageError::new("serialize", LEGACY_ACTIVITY_TYPES, e.to_string()))?;
    target
        .inner()
        .set_item(LEGACY_ACTIVITY_TYPES, &serialized)
        .await?;

    Ok(Some(types.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{block_on, read_json_fixture};
    use crate::storage::MemStorage;

    fn fixture() -> serde_json::Value {
        read_json_fixture("legacy-migration.json")
    }

    fn legacy_store() -> MemStorage {
        let f = fixture();
        let legacy = f["legacyLocalStorage"].as_object().expect("legacy keys");
        MemStorage::seeded(
            legacy
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().expect("string").to_owned())),
        )
    }

    #[test]
    fn migrates_a_pre_indexeddb_profile_exactly_as_the_csharp_did() {
        block_on(async {
            let legacy = legacy_store();
            let target = WeekBucketed::new(MemStorage::new());

            let report = migrate_from_legacy(&legacy, &target).await;
            assert!(report.failures.is_empty(), "{:?}", report.failures);

            let expected = fixture();
            let expected_writes = expected["indexedDbWritesAfterMigration"]
                .as_object()
                .expect("writes");

            let actual = target.inner().snapshot();
            assert_eq!(
                actual.keys().cloned().collect::<Vec<_>>(),
                expected_writes.keys().cloned().collect::<Vec<_>>(),
                "migrated keys must match the C# run"
            );

            for (key, value) in expected_writes {
                assert_eq!(
                    actual.get(key).map(String::as_str),
                    Some(value.as_str().expect("string")),
                    "content diverged for {key}"
                );
            }
        });
    }

    #[test]
    fn splits_across_the_year_boundary() {
        block_on(async {
            let legacy = legacy_store();
            let target = WeekBucketed::new(MemStorage::new());
            let report = migrate_from_legacy(&legacy, &target).await;

            assert!(
                report
                    .buckets_written
                    .contains(&"activities-2025.53".to_owned())
            );
            assert!(
                report
                    .buckets_written
                    .contains(&"activities-2026.01".to_owned())
            );
            assert_eq!(report.activities_migrated, 4);
            assert_eq!(report.activity_types_migrated, 2);
        });
    }

    #[test]
    fn removes_both_legacy_keys_only_after_writing() {
        block_on(async {
            let legacy = legacy_store();
            let target = WeekBucketed::new(MemStorage::new());
            let report = migrate_from_legacy(&legacy, &target).await;

            let expected: Vec<String> = fixture()["localStorageKeysRemoved"]
                .as_array()
                .expect("array")
                .iter()
                .map(|v| v.as_str().expect("string").to_owned())
                .collect();

            assert_eq!(report.legacy_keys_removed, expected);
            assert!(legacy.is_empty(), "legacy keys must be gone afterwards");
        });
    }

    #[test]
    fn activity_types_are_written_unbucketed() {
        block_on(async {
            let legacy = legacy_store();
            let target = WeekBucketed::new(MemStorage::new());
            migrate_from_legacy(&legacy, &target).await;

            assert!(target.inner().contains("activityTypes"));
            // It must not be mistaken for a week bucket.
            assert!(
                !target
                    .available_week_keys()
                    .await
                    .expect("ok")
                    .iter()
                    .any(|k| k == "Types")
            );
        });
    }

    #[test]
    fn an_empty_profile_migrates_nothing() {
        block_on(async {
            let legacy = MemStorage::new();
            let target = WeekBucketed::new(MemStorage::new());

            let report = migrate_from_legacy(&legacy, &target).await;
            assert!(!report.migrated_anything());
            assert!(report.failures.is_empty());
            assert!(target.inner().is_empty());
        });
    }

    #[test]
    fn an_empty_legacy_list_migrates_nothing_and_leaves_the_key() {
        block_on(async {
            let legacy = MemStorage::seeded([("activities", "[]")]);
            let target = WeekBucketed::new(MemStorage::new());

            let report = migrate_from_legacy(&legacy, &target).await;
            assert!(!report.migrated_anything());
            assert!(target.inner().is_empty());
            // The C# only removes the key after a successful non-empty write.
            assert!(legacy.contains("activities"));
        });
    }

    #[test]
    fn corrupt_legacy_data_is_reported_but_does_not_stop_startup() {
        block_on(async {
            let legacy = MemStorage::seeded([("activities", "{not json"), ("activityTypes", "[]")]);
            let target = WeekBucketed::new(MemStorage::new());

            let report = migrate_from_legacy(&legacy, &target).await;

            assert_eq!(report.failures.len(), 1, "the parse failure is recorded");
            assert_eq!(report.activities_migrated, 0);
            // The corrupt data is left in place rather than silently discarded.
            assert!(legacy.contains("activities"));
        });
    }

    #[test]
    fn a_second_run_is_a_no_op() {
        block_on(async {
            let legacy = legacy_store();
            let target = WeekBucketed::new(MemStorage::new());

            let first = migrate_from_legacy(&legacy, &target).await;
            assert!(first.migrated_anything());
            let after_first = target.inner().snapshot();

            let second = migrate_from_legacy(&legacy, &target).await;
            assert!(!second.migrated_anything());
            assert_eq!(target.inner().snapshot(), after_first);
        });
    }
}
