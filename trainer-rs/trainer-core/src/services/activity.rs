//! Activities, ported from `Trainer/Services/ActivityService.cs`.
//!
//! Activities live in week buckets, so this service works over
//! [`WeekBucketed`] rather than a bare store: adding, updating and deleting all
//! need the individual bucket, while listing goes through the aggregate view.

use crate::models::Activity;
use crate::storage::buckets::WeekBucketed;
use crate::storage::{Storage, StorageError, StorageResult};
use crate::week;
use chrono::NaiveDateTime;
use std::cell::Cell;

const ACTIVITIES_KEY: &str = "activities";
const NEXT_ID_KEY: &str = "activityNextId";

pub struct ActivityService<'a, S: Storage> {
    storage: &'a WeekBucketed<S>,
    next_id: Cell<i32>,
    next_id_initialized: Cell<bool>,
}

impl<'a, S: Storage> ActivityService<'a, S> {
    pub fn new(storage: &'a WeekBucketed<S>) -> Self {
        Self {
            storage,
            next_id: Cell::new(1),
            next_id_initialized: Cell::new(false),
        }
    }

    async fn all_activities(&self) -> StorageResult<Vec<Activity>> {
        let json = self
            .storage
            .get_item(ACTIVITIES_KEY)
            .await?
            .unwrap_or_else(|| "[]".to_owned());
        serde_json::from_str(&json)
            .map_err(|e| StorageError::new("deserialize", ACTIVITIES_KEY, e.to_string()))
    }

    async fn persist_next_id(&self, value: i32) -> StorageResult<()> {
        // Stored as a bare JSON number, matching how the profile holds it.
        self.storage.set_item(NEXT_ID_KEY, &value.to_string()).await
    }

    /// Loads the next id, or derives it from existing activities on a profile
    /// that predates the stored counter.
    async fn ensure_next_id(&self) -> StorageResult<()> {
        if self.next_id_initialized.get() {
            return Ok(());
        }

        let stored: Option<i32> = match self.storage.get_item(NEXT_ID_KEY).await? {
            Some(json) => serde_json::from_str(&json).ok(),
            None => None,
        };

        match stored {
            Some(value) if value > 0 => self.next_id.set(value),
            _ => {
                let activities = self.all_activities().await?;
                let derived = activities
                    .iter()
                    .map(|a| a.id)
                    .max()
                    .map_or(1, |max| max + 1);
                self.next_id.set(derived);
                self.persist_next_id(derived).await?;
            }
        }

        self.next_id_initialized.set(true);
        Ok(())
    }

    /// Activities, newest first.
    ///
    /// **No date filtering happens here**, matching the C#: a date range is
    /// resolved to week keys and those buckets are returned whole. Callers that
    /// want an exact range must filter themselves.
    pub async fn all(
        &self,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
    ) -> StorageResult<Vec<Activity>> {
        let mut activities = if start.is_some() || end.is_some() {
            let from = start.unwrap_or(NaiveDateTime::MIN);
            let to = end.unwrap_or(NaiveDateTime::MAX);
            let week_keys = week::week_keys_in_range(from, to);
            self.storage.activities_for_weeks(&week_keys).await?
        } else {
            self.all_activities().await?
        };

        self.ensure_next_id().await?;

        // Descending by wall clock, as the C# orders by the DateTime value.
        // `sort_by_key` is a stable sort, as LINQ's OrderByDescending is, so
        // activities sharing a timestamp keep their storage order.
        activities.sort_by_key(|a| std::cmp::Reverse(a.when.naive()));
        Ok(activities)
    }

    pub async fn by_id(&self, id: i32) -> StorageResult<Option<Activity>> {
        Ok(self.all(None, None).await?.into_iter().find(|a| a.id == id))
    }

    pub async fn by_activity_type_id(&self, activity_type_id: i32) -> StorageResult<Vec<Activity>> {
        Ok(self
            .all(None, None)
            .await?
            .into_iter()
            .filter(|a| a.activity_type_id == activity_type_id)
            .collect())
    }

    pub async fn available_week_keys(&self) -> StorageResult<Vec<String>> {
        self.storage.available_week_keys().await
    }

    /// Appends an activity, assigning it the next id.
    pub async fn add(&self, mut activity: Activity) -> StorageResult<Activity> {
        self.ensure_next_id().await?;

        activity.id = self.next_id.get();
        self.next_id.set(activity.id + 1);
        self.persist_next_id(self.next_id.get()).await?;

        let week_key = week::week_key(activity.when.naive());
        let mut bucket = self.storage.activities_for_week(&week_key).await?;
        bucket.push(activity.clone());
        self.storage
            .set_activities_for_week(&week_key, &bucket)
            .await?;

        Ok(activity)
    }

    /// Replaces an activity, moving it between buckets if its week changed.
    ///
    /// An activity that no longer exists is ignored.
    pub async fn update(&self, activity: Activity) -> StorageResult<()> {
        let Some(existing) = self.by_id(activity.id).await? else {
            return Ok(());
        };

        let old_week = week::week_key(existing.when.naive());
        let new_week = week::week_key(activity.when.naive());

        if old_week == new_week {
            let mut bucket = self.storage.activities_for_week(&old_week).await?;
            bucket.retain(|a| a.id != activity.id);
            bucket.push(activity);
            self.storage
                .set_activities_for_week(&old_week, &bucket)
                .await?;
            return Ok(());
        }

        let mut old_bucket = self.storage.activities_for_week(&old_week).await?;
        old_bucket.retain(|a| a.id != activity.id);
        if old_bucket.is_empty() {
            self.storage.remove_activities_for_week(&old_week).await?;
        } else {
            self.storage
                .set_activities_for_week(&old_week, &old_bucket)
                .await?;
        }

        let mut new_bucket = self.storage.activities_for_week(&new_week).await?;
        new_bucket.retain(|a| a.id != activity.id);
        new_bucket.push(activity);
        self.storage
            .set_activities_for_week(&new_week, &new_bucket)
            .await
    }

    /// Removes an activity.
    ///
    /// **Asymmetry preserved from the C#:** unlike [`Self::update`], this leaves
    /// an emptied bucket behind as an empty array rather than deleting the key.
    /// Harmless, since reads flatten empty buckets away, but it is why a profile
    /// can hold `activities-*` keys containing `[]`.
    pub async fn delete(&self, id: i32) -> StorageResult<()> {
        let Some(activity) = self.by_id(id).await? else {
            return Ok(());
        };

        let week_key = week::week_key(activity.when.naive());
        let mut bucket = self.storage.activities_for_week(&week_key).await?;
        bucket.retain(|a| a.id != id);
        self.storage
            .set_activities_for_week(&week_key, &bucket)
            .await
    }

    /// Recomputes the next id from every stored activity. Called after an
    /// import, so restored ids cannot collide with newly assigned ones.
    pub async fn recalculate_next_id(&self) -> StorageResult<()> {
        let activities = self.all_activities().await?;
        let derived = activities
            .iter()
            .map(|a| a.id)
            .max()
            .map_or(1, |max| max + 1);
        self.next_id.set(derived);
        self.persist_next_id(derived).await?;
        self.next_id_initialized.set(true);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::TrainerTime;
    use crate::fixtures::block_on;
    use crate::storage::MemStorage;

    fn at(text: &str) -> TrainerTime {
        TrainerTime::parse(text).expect("valid timestamp")
    }

    fn activity(id: i32, type_id: i32, when: &str) -> Activity {
        Activity {
            id,
            activity_type_id: type_id,
            when: at(when),
            amount: 1,
            notes: None,
            duration_seconds: None,
            known_location_id: None,
        }
    }

    fn store() -> WeekBucketed<MemStorage> {
        WeekBucketed::new(MemStorage::new())
    }

    /// Ports `GetAllAsync_ReturnsEmptyList_WhenNoActivitiesExist`.
    #[test]
    fn returns_empty_when_nothing_is_stored() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            assert!(service.all(None, None).await.expect("ok").is_empty());
        });
    }

    /// Ports `GetAllAsync_ReturnsAllActivities`, including the ordering.
    #[test]
    fn returns_all_activities_newest_first() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);

            service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            service
                .add(activity(0, 1, "2026-02-01T08:00:00-08"))
                .await
                .expect("ok");
            service
                .add(activity(0, 1, "2026-01-15T08:00:00-08"))
                .await
                .expect("ok");

            let all = service.all(None, None).await.expect("ok");
            let dates: Vec<String> = all
                .iter()
                .map(|a| a.when.naive().date().to_string())
                .collect();
            assert_eq!(dates, vec!["2026-02-01", "2026-01-15", "2026-01-01"]);
        });
    }

    /// Ports `AddAsync_AddsActivityWithNewId` and `AddAsync_PersistsNextId_AfterIncrementing`.
    #[test]
    fn add_assigns_and_persists_sequential_ids() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);

            let first = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            let second = service
                .add(activity(0, 1, "2026-01-02T08:00:00-08"))
                .await
                .expect("ok");

            assert_eq!(first.id, 1);
            assert_eq!(second.id, 2);
            assert_eq!(
                storage
                    .inner()
                    .snapshot()
                    .get(NEXT_ID_KEY)
                    .map(String::as_str),
                Some("3"),
                "the counter is persisted after each add"
            );
        });
    }

    /// Ports `GetByIdAsync_ReturnsActivity_WhenExists` / `_WhenNotExists`.
    #[test]
    fn finds_by_id_or_returns_none() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            let added = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");

            assert_eq!(
                service.by_id(added.id).await.expect("ok").map(|a| a.id),
                Some(added.id)
            );
            assert!(service.by_id(999).await.expect("ok").is_none());
        });
    }

    /// Ports `UpdateAsync_UpdatesExistingActivity`.
    #[test]
    fn update_replaces_within_the_same_week() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            let mut added = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");

            added.amount = 42;
            added.notes = Some("edited".to_owned());
            service.update(added.clone()).await.expect("ok");

            let all = service.all(None, None).await.expect("ok");
            assert_eq!(all.len(), 1, "updated, not duplicated");
            assert_eq!(all[0].amount, 42);
            assert_eq!(all[0].notes.as_deref(), Some("edited"));
        });
    }

    #[test]
    fn update_moves_an_activity_between_week_buckets() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            let mut added = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            assert_eq!(
                storage.available_week_keys().await.expect("ok"),
                vec!["2026.01"]
            );

            // Move it into a different week.
            added.when = at("2026-03-05T08:00:00-08");
            service.update(added).await.expect("ok");

            let weeks = storage.available_week_keys().await.expect("ok");
            assert_eq!(weeks, vec!["2026.10"], "the emptied bucket is deleted");
            assert_eq!(service.all(None, None).await.expect("ok").len(), 1);
        });
    }

    #[test]
    fn update_moving_out_of_a_shared_week_keeps_the_bucket() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            let mut second = service
                .add(activity(0, 1, "2026-01-02T08:00:00-08"))
                .await
                .expect("ok");

            second.when = at("2026-03-05T08:00:00-08");
            service.update(second).await.expect("ok");

            let mut weeks = storage.available_week_keys().await.expect("ok");
            weeks.sort();
            assert_eq!(weeks, vec!["2026.01", "2026.10"]);
        });
    }

    #[test]
    fn updating_an_absent_activity_is_ignored() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            service
                .update(activity(999, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            assert!(service.all(None, None).await.expect("ok").is_empty());
        });
    }

    /// Ports `DeleteAsync_RemovesActivity`.
    #[test]
    fn delete_removes_the_activity() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            let first = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            service
                .add(activity(0, 1, "2026-01-02T08:00:00-08"))
                .await
                .expect("ok");

            service.delete(first.id).await.expect("ok");

            let all = service.all(None, None).await.expect("ok");
            assert_eq!(all.len(), 1);
            assert_ne!(all[0].id, first.id);
        });
    }

    #[test]
    fn delete_leaves_an_emptied_bucket_behind_unlike_update() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            let only = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");

            service.delete(only.id).await.expect("ok");

            // The C# does not remove the key here, only in update. Preserved.
            assert!(storage.inner().contains("activities-2026.01"));
            assert!(service.all(None, None).await.expect("ok").is_empty());
        });
    }

    /// Ports `GetByActivityTypeIdAsync_ReturnsFilteredActivities`.
    #[test]
    fn filters_by_activity_type() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            service
                .add(activity(0, 2, "2026-01-02T08:00:00-08"))
                .await
                .expect("ok");
            service
                .add(activity(0, 1, "2026-01-03T08:00:00-08"))
                .await
                .expect("ok");

            let filtered = service.by_activity_type_id(1).await.expect("ok");
            assert_eq!(filtered.len(), 2);
            assert!(filtered.iter().all(|a| a.activity_type_id == 1));
        });
    }

    /// Ports `EnsureNextIdInitializedAsync_LoadsFromLocalStorage_WhenExists`.
    #[test]
    fn the_stored_counter_is_used_when_present() {
        block_on(async {
            let storage = WeekBucketed::new(MemStorage::seeded([(NEXT_ID_KEY, "42")]));
            let service = ActivityService::new(&storage);

            let added = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            assert_eq!(added.id, 42);
        });
    }

    /// Ports `EnsureNextIdInitializedAsync_CalculatesFromActivities_WhenLocalStorageEmpty`.
    #[test]
    fn the_counter_is_derived_from_activities_when_absent() {
        block_on(async {
            let storage = store();
            // Seed a bucket directly, without the counter.
            storage
                .set_activities_for_week("2026.01", &vec![activity(7, 1, "2026-01-01T08:00:00-08")])
                .await
                .expect("ok");

            let service = ActivityService::new(&storage);
            let added = service
                .add(activity(0, 1, "2026-01-02T08:00:00-08"))
                .await
                .expect("ok");
            assert_eq!(added.id, 8, "max existing id plus one");
        });
    }

    /// Ports `EnsureNextIdInitializedAsync_DefaultsToOne_WhenNoActivitiesAndNoLocalStorage`.
    #[test]
    fn the_counter_defaults_to_one_on_an_empty_profile() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            service.all(None, None).await.expect("ok");
            assert_eq!(
                storage
                    .inner()
                    .snapshot()
                    .get(NEXT_ID_KEY)
                    .map(String::as_str),
                Some("1")
            );
        });
    }

    #[test]
    fn a_non_positive_stored_counter_is_ignored() {
        block_on(async {
            let storage = WeekBucketed::new(MemStorage::seeded([(NEXT_ID_KEY, "0")]));
            let service = ActivityService::new(&storage);
            let added = service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            assert_eq!(added.id, 1);
        });
    }

    /// Ports `RecalculateNextIdAsync_UpdatesNextId_FromAllActivities` and `_SetsToOne_WhenNoActivities`.
    #[test]
    fn recalculating_derives_from_every_stored_activity() {
        block_on(async {
            let storage = store();
            storage
                .set_activities_for_week(
                    "2026.01",
                    &vec![
                        activity(3, 1, "2026-01-01T08:00:00-08"),
                        activity(11, 1, "2026-01-02T08:00:00-08"),
                    ],
                )
                .await
                .expect("ok");

            let service = ActivityService::new(&storage);
            service.recalculate_next_id().await.expect("ok");
            assert_eq!(
                storage
                    .inner()
                    .snapshot()
                    .get(NEXT_ID_KEY)
                    .map(String::as_str),
                Some("12")
            );

            let empty = store();
            let empty_service = ActivityService::new(&empty);
            empty_service.recalculate_next_id().await.expect("ok");
            assert_eq!(
                empty
                    .inner()
                    .snapshot()
                    .get(NEXT_ID_KEY)
                    .map(String::as_str),
                Some("1")
            );
        });
    }

    /// Ports `AddAsync_AfterImport_RecalculatesNextId`.
    #[test]
    fn adding_after_an_import_does_not_collide_with_restored_ids() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            service.all(None, None).await.expect("ok"); // counter starts at 1

            // Simulate an import writing high ids straight into storage.
            storage
                .set_activities_for_week(
                    "2026.01",
                    &vec![activity(500, 1, "2026-01-01T08:00:00-08")],
                )
                .await
                .expect("ok");
            service.recalculate_next_id().await.expect("ok");

            let added = service
                .add(activity(0, 1, "2026-01-02T08:00:00-08"))
                .await
                .expect("ok");
            assert_eq!(added.id, 501);
        });
    }

    #[test]
    fn a_date_range_returns_whole_buckets_without_filtering() {
        block_on(async {
            let storage = store();
            let service = ActivityService::new(&storage);
            service
                .add(activity(0, 1, "2026-01-01T08:00:00-08"))
                .await
                .expect("ok");
            service
                .add(activity(0, 1, "2026-01-04T08:00:00-08"))
                .await
                .expect("ok");

            // Both fall in bucket 2026.01, so asking for only the first day
            // still returns both. This is the C# behavior, preserved.
            let ranged = service
                .all(
                    Some(at("2026-01-01T00:00:00-08").naive()),
                    Some(at("2026-01-01T23:59:59-08").naive()),
                )
                .await
                .expect("ok");
            assert_eq!(ranged.len(), 2, "no date filtering is applied");
        });
    }
}
