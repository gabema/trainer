//! Week bucketing, ported from the special handling inside
//! `Trainer/Services/IndexedDbStorageService.cs`.
//!
//! Activities are not stored under one key. They are split by week into
//! `activities-{weekKey}` buckets, and the logical `activities` key is a view
//! that aggregates them. [`WeekBucketed`] is a decorator implementing
//! [`Storage`], so services see the interface the C# services saw while the
//! bucketing itself stays testable without a browser.

use super::{Storage, StorageError, StorageResult};
use crate::models::{self, Activity, Format};
use crate::week;
use async_trait::async_trait;
use std::collections::{BTreeMap, BTreeSet};

/// The logical key that aggregates every week bucket.
pub const ACTIVITIES_KEY: &str = "activities";
/// Prefix shared by all week buckets.
pub const ACTIVITIES_PREFIX: &str = "activities-";

/// Wraps a raw store, adding the `activities` aggregate view and week bucketing.
#[derive(Debug, Default)]
pub struct WeekBucketed<S> {
    inner: S,
}

impl<S: Storage> WeekBucketed<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    /// The underlying store, for assertions and raw access.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    fn parse_activities(key: &str, json: &str) -> StorageResult<Vec<Activity>> {
        serde_json::from_str(json).map_err(|e| StorageError::new("deserialize", key, e.to_string()))
    }

    fn write_activities(key: &str, activities: &Vec<Activity>) -> StorageResult<String> {
        models::to_json(activities, Format::Storage)
            .map_err(|e| StorageError::new("serialize", key, e.to_string()))
    }

    /// Every week key currently present, recovered from the storage keys.
    pub async fn available_week_keys(&self) -> StorageResult<Vec<String>> {
        let storage_keys = self.inner.keys_with_prefix(ACTIVITIES_PREFIX).await?;
        Ok(storage_keys
            .iter()
            .filter_map(|k| week::extract_week_key(k).ok())
            .map(str::to_owned)
            .collect())
    }

    pub async fn activities_for_week(&self, week_key: &str) -> StorageResult<Vec<Activity>> {
        let storage_key = week::storage_key(week_key);
        match self.inner.get_item(&storage_key).await? {
            Some(json) => Self::parse_activities(&storage_key, &json),
            None => Ok(Vec::new()),
        }
    }

    /// Activities across several weeks, flattened.
    ///
    /// Buckets are visited in sorted key order, which is chronological because
    /// week keys are zero-padded. The C# iterates a dictionary built from
    /// completed JS requests, so its order is arbitrary; every caller sorts the
    /// result afterwards, so determinism here is a strict improvement.
    pub async fn activities_for_weeks(&self, week_keys: &[String]) -> StorageResult<Vec<Activity>> {
        let storage_keys: Vec<String> = week_keys.iter().map(|k| week::storage_key(k)).collect();
        let items = self.inner.get_items(&storage_keys).await?;

        let mut all = Vec::new();
        for (key, json) in items {
            all.extend(Self::parse_activities(&key, &json)?);
        }
        Ok(all)
    }

    pub async fn set_activities_for_week(
        &self,
        week_key: &str,
        activities: &Vec<Activity>,
    ) -> StorageResult<()> {
        let storage_key = week::storage_key(week_key);
        let json = Self::write_activities(&storage_key, activities)?;
        self.inner.set_item(&storage_key, &json).await
    }

    pub async fn remove_activities_for_week(&self, week_key: &str) -> StorageResult<()> {
        self.inner.remove_item(&week::storage_key(week_key)).await
    }

    /// Groups a flat list into buckets, writes each, and removes any bucket
    /// that no longer holds activities — matching `SetActivitiesAsync`.
    async fn replace_all_activities(&self, activities: Vec<Activity>) -> StorageResult<()> {
        let mut by_week: BTreeMap<String, Vec<Activity>> = BTreeMap::new();
        for activity in activities {
            by_week
                .entry(week::week_key(activity.when.naive()))
                .or_default()
                .push(activity);
        }

        let mut stale: BTreeSet<String> = self
            .inner
            .keys_with_prefix(ACTIVITIES_PREFIX)
            .await?
            .into_iter()
            .collect();

        for (week_key, bucket) in &by_week {
            let storage_key = week::storage_key(week_key);
            let json = Self::write_activities(&storage_key, bucket)?;
            self.inner.set_item(&storage_key, &json).await?;
            stale.remove(&storage_key);
        }

        for empty_key in stale {
            self.inner.remove_item(&empty_key).await?;
        }

        Ok(())
    }

    /// Flattens every bucket into the aggregate JSON array.
    async fn all_activities_json(&self) -> StorageResult<String> {
        let storage_keys = self.inner.keys_with_prefix(ACTIVITIES_PREFIX).await?;
        if storage_keys.is_empty() {
            return Ok("[]".to_owned());
        }

        let items = self.inner.get_items(&storage_keys).await?;
        let mut all = Vec::new();
        for (key, json) in items {
            all.extend(Self::parse_activities(&key, &json)?);
        }

        Self::write_activities(ACTIVITIES_KEY, &all)
    }
}

#[async_trait(?Send)]
impl<S: Storage> Storage for WeekBucketed<S> {
    async fn get_item(&self, key: &str) -> StorageResult<Option<String>> {
        if key == ACTIVITIES_KEY {
            return Ok(Some(self.all_activities_json().await?));
        }
        self.inner.get_item(key).await
    }

    async fn set_item(&self, key: &str, value: &str) -> StorageResult<()> {
        if key == ACTIVITIES_KEY {
            let activities = Self::parse_activities(ACTIVITIES_KEY, value)?;
            return self.replace_all_activities(activities).await;
        }
        self.inner.set_item(key, value).await
    }

    async fn remove_item(&self, key: &str) -> StorageResult<()> {
        if key == ACTIVITIES_KEY {
            for storage_key in self.inner.keys_with_prefix(ACTIVITIES_PREFIX).await? {
                self.inner.remove_item(&storage_key).await?;
            }
            return Ok(());
        }
        self.inner.remove_item(key).await
    }

    async fn clear(&self) -> StorageResult<()> {
        self.inner.clear().await
    }

    async fn keys_with_prefix(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.inner.keys_with_prefix(prefix).await
    }

    async fn get_items(&self, keys: &[String]) -> StorageResult<BTreeMap<String, String>> {
        self.inner.get_items(keys).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{block_on, read_json_fixture};
    use crate::storage::MemStorage;

    /// Rebuilds a store from the real de-identified IndexedDB dump, so the
    /// bucketing layer runs against genuine key layout and content.
    fn store_from_snapshot() -> WeekBucketed<MemStorage> {
        let snapshot = read_json_fixture("idb-snapshot.json");
        let entries = snapshot["entries"].as_object().expect("entries");

        let mem = MemStorage::new();
        for (key, entry) in entries {
            let json = serde_json::to_string(&entry["value"]).expect("serializes");
            block_on(mem.set_item(key, &json)).expect("seeded");
        }
        WeekBucketed::new(mem)
    }

    #[test]
    fn aggregate_key_flattens_every_bucket() {
        block_on(async {
            let store = store_from_snapshot();
            let json = store
                .get_item(ACTIVITIES_KEY)
                .await
                .expect("ok")
                .expect("aggregate is always present");

            let activities: Vec<Activity> = serde_json::from_str(&json).expect("parses");
            assert_eq!(
                activities.len(),
                527,
                "the real profile holds 527 activities"
            );
        });
    }

    #[test]
    fn aggregate_key_is_an_empty_array_when_nothing_is_stored() {
        block_on(async {
            let store = WeekBucketed::new(MemStorage::new());
            assert_eq!(
                store.get_item(ACTIVITIES_KEY).await.expect("ok"),
                Some("[]".to_owned())
            );
        });
    }

    #[test]
    fn week_keys_are_recovered_from_storage_keys() {
        block_on(async {
            let store = store_from_snapshot();
            let mut keys = store.available_week_keys().await.expect("ok");
            keys.sort();

            assert_eq!(keys.len(), 31);
            assert_eq!(keys.first().map(String::as_str), Some("2026.01"));
            // activityTypes / knownLocations / activityNextId must not appear.
            assert!(keys.iter().all(|k| !k.starts_with("activit")));
        });
    }

    #[test]
    fn writing_the_aggregate_regroups_into_buckets() {
        block_on(async {
            let store = WeekBucketed::new(MemStorage::new());
            let source = read_json_fixture("export.json");
            let mut all = Vec::new();
            for bucket in source["activities"].as_object().expect("buckets").values() {
                let parsed: Vec<Activity> = serde_json::from_value(bucket.clone()).expect("parses");
                all.extend(parsed);
            }
            let flat = models::to_json(&all, Format::Storage).expect("serializes");

            store.set_item(ACTIVITIES_KEY, &flat).await.expect("ok");

            let mut keys = store
                .inner()
                .keys_with_prefix(ACTIVITIES_PREFIX)
                .await
                .expect("ok");
            keys.sort();
            assert_eq!(keys.len(), 31, "regrouped into the same 31 buckets");
            assert!(keys.contains(&"activities-2026.01".to_owned()));
        });
    }

    #[test]
    fn emptied_buckets_are_removed_rather_than_left_as_empty_arrays() {
        block_on(async {
            let store = store_from_snapshot();
            let before = store.available_week_keys().await.expect("ok").len();
            assert!(before > 1);

            // Keep only one week's activities and rewrite the aggregate.
            let kept = store.activities_for_week("2026.01").await.expect("ok");
            assert!(!kept.is_empty());
            let json = models::to_json(&kept, Format::Storage).expect("serializes");
            store.set_item(ACTIVITIES_KEY, &json).await.expect("ok");

            let after = store.available_week_keys().await.expect("ok");
            assert_eq!(after, vec!["2026.01"], "stale buckets must be deleted");
            assert!(!store.inner().contains("activities-2026.02"));
        });
    }

    #[test]
    fn removing_the_aggregate_removes_every_bucket_but_nothing_else() {
        block_on(async {
            let store = store_from_snapshot();
            store.remove_item(ACTIVITIES_KEY).await.expect("ok");

            assert!(store.available_week_keys().await.expect("ok").is_empty());
            // Sibling keys sharing the "activit" prefix must survive.
            assert!(store.inner().contains("activityTypes"));
            assert!(store.inner().contains("activityNextId"));
            assert!(store.inner().contains("knownLocations"));
        });
    }

    #[test]
    fn non_activity_keys_pass_straight_through() {
        block_on(async {
            let store = WeekBucketed::new(MemStorage::new());
            store.set_item("activityNextId", "536").await.expect("ok");
            assert_eq!(
                store.get_item("activityNextId").await.expect("ok"),
                Some("536".to_owned())
            );

            store.remove_item("activityNextId").await.expect("ok");
            assert_eq!(store.get_item("activityNextId").await.expect("ok"), None);
        });
    }

    #[test]
    fn week_range_reads_flatten_across_buckets() {
        block_on(async {
            let store = store_from_snapshot();
            let both = store
                .activities_for_weeks(&["2026.01".to_owned(), "2026.02".to_owned()])
                .await
                .expect("ok");

            let first = store.activities_for_week("2026.01").await.expect("ok");
            let second = store.activities_for_week("2026.02").await.expect("ok");
            assert_eq!(both.len(), first.len() + second.len());
        });
    }

    #[test]
    fn a_missing_week_reads_as_empty_rather_than_failing() {
        block_on(async {
            let store = store_from_snapshot();
            assert!(
                store
                    .activities_for_week("1999.01")
                    .await
                    .expect("ok")
                    .is_empty()
            );
        });
    }

    #[test]
    fn corrupt_bucket_content_surfaces_as_an_error() {
        block_on(async {
            let mem = MemStorage::seeded([("activities-2026.01", "{not json")]);
            let store = WeekBucketed::new(mem);
            let err = store
                .activities_for_week("2026.01")
                .await
                .expect_err("should fail");
            assert_eq!(err.operation, "deserialize");
        });
    }
}
