//! In-progress activity timers, ported from
//! `Trainer/Services/ActiveActivityService.cs`.
//!
//! State lives in localStorage under `trainer_active_activities` as a list of
//! `{id, startTime}` entries. Start times use [`ActiveTime`], not the format
//! activities use — see that module for why.
//!
//! # Notifications
//!
//! The C# exposes `OnChanged`, `OnTick` and `OnSlowTick` events plus one- and
//! thirty-second timers, and every consumer subscribes, unsubscribes and calls
//! `StateHasChanged` by hand. That is a view concern: `rust-ui` replaces it with
//! signals, and the timers belong to the component that renders elapsed time.
//! What this service owns is the state and a [`version`](Self::version) counter
//! the view can observe.

use super::active_time::ActiveTime;
use crate::escaping;
use crate::storage::{Storage, StorageResult};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

const STORAGE_KEY: &str = "trainer_active_activities";

/// One persisted entry. Field names are fixed by `JsonPropertyName` attributes
/// on the C# record, so they stay camelCase regardless of naming policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredEntry {
    id: i32,
    #[serde(rename = "startTime")]
    start_time: ActiveTime,
}

pub struct ActiveActivityService<'a, S: Storage> {
    storage: &'a S,
    active: RefCell<BTreeMap<i32, ActiveTime>>,
    version: Cell<u64>,
}

impl<'a, S: Storage> ActiveActivityService<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        Self {
            storage,
            active: RefCell::new(BTreeMap::new()),
            version: Cell::new(0),
        }
    }

    /// Increments whenever the active set changes, so a view can react without
    /// the subscribe/unsubscribe bookkeeping the C# required.
    pub fn version(&self) -> u64 {
        self.version.get()
    }

    fn changed(&self) {
        self.version.set(self.version.get() + 1);
    }

    /// Loads persisted state. Corrupt state is cleared and treated as empty
    /// rather than surfacing an error, matching the C# recovery.
    pub async fn initialize(&self) -> StorageResult<()> {
        let Some(json) = self.storage.get_item(STORAGE_KEY).await? else {
            return Ok(());
        };
        if json.is_empty() {
            return Ok(());
        }

        match serde_json::from_str::<Vec<StoredEntry>>(&json) {
            Ok(entries) if !entries.is_empty() => {
                let mut active = self.active.borrow_mut();
                for entry in entries {
                    active.insert(entry.id, entry.start_time);
                }
                drop(active);
                self.changed();
            }
            Ok(_) => {}
            Err(_) => {
                // Corrupt stored data: clear it and start fresh.
                self.storage.remove_item(STORAGE_KEY).await?;
            }
        }
        Ok(())
    }

    pub fn is_active(&self, activity_id: i32) -> bool {
        self.active.borrow().contains_key(&activity_id)
    }

    pub fn all(&self) -> BTreeMap<i32, ActiveTime> {
        self.active.borrow().clone()
    }

    pub fn len(&self) -> usize {
        self.active.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.active.borrow().is_empty()
    }

    pub async fn start(&self, activity_id: i32, start_time: ActiveTime) -> StorageResult<()> {
        self.active.borrow_mut().insert(activity_id, start_time);
        self.changed();
        self.persist().await
    }

    pub async fn finish(&self, activity_id: i32) -> StorageResult<()> {
        self.active.borrow_mut().remove(&activity_id);
        self.changed();
        self.persist().await
    }

    /// Writes the current set, or removes the key entirely when nothing is
    /// active — the C# removes rather than storing an empty array.
    async fn persist(&self) -> StorageResult<()> {
        let entries: Vec<StoredEntry> = self
            .active
            .borrow()
            .iter()
            .map(|(id, start_time)| StoredEntry {
                id: *id,
                start_time: *start_time,
            })
            .collect();

        if entries.is_empty() {
            return self.storage.remove_item(STORAGE_KEY).await;
        }

        // Default JsonSerializerOptions still uses JavaScriptEncoder.Default, so
        // a positive offset's "+" is escaped here exactly as elsewhere.
        let json = escaping::to_string(&entries).map_err(|e| {
            crate::storage::StorageError::new("serialize", STORAGE_KEY, e.to_string())
        })?;
        self.storage.set_item(STORAGE_KEY, &json).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{block_on, read_json_fixture};
    use crate::storage::MemStorage;

    fn at(text: &str) -> ActiveTime {
        ActiveTime::parse(text).expect("valid")
    }

    /// Ports `IsActive_BeforeStart_ReturnsFalse` and `Start_MakesActivityActive`.
    #[test]
    fn starting_marks_an_activity_active() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);

            assert!(!service.is_active(1));
            service
                .start(1, at("2026-08-28T15:43:21-07:00"))
                .await
                .expect("ok");
            assert!(service.is_active(1));
        });
    }

    /// Ports `Start_RecordsSuppliedStartTime`.
    #[test]
    fn the_supplied_start_time_is_recorded() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);
            let when = at("2026-06-01T10:00:00-07:00");

            service.start(1, when).await.expect("ok");
            assert_eq!(service.all().get(&1), Some(&when));
        });
    }

    /// Ports `Finish_RemovesActivity` and `Finish_NonExistentActivity_DoesNotThrow`.
    #[test]
    fn finishing_removes_an_activity_and_tolerates_unknown_ids() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);

            service
                .start(1, at("2026-08-28T15:43:21-07:00"))
                .await
                .expect("ok");
            service.finish(1).await.expect("ok");
            assert!(!service.is_active(1));

            service.finish(999).await.expect("ok");
        });
    }

    /// Ports `GetAll_ReturnsAllActiveActivities` and
    /// `Start_MultipleActivities_EachTrackedIndependently`.
    #[test]
    fn activities_are_tracked_independently() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);

            service
                .start(10, at("2026-08-28T15:43:21-07:00"))
                .await
                .expect("ok");
            service
                .start(20, at("2026-08-28T16:00:00-07:00"))
                .await
                .expect("ok");
            assert_eq!(service.all().len(), 2);

            service.finish(10).await.expect("ok");
            assert!(!service.is_active(10));
            assert!(service.is_active(20));
        });
    }

    /// Ports `Start_RaisesOnChanged` / `Finish_RaisesOnChanged`, as a version bump.
    #[test]
    fn changes_bump_the_version() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);

            let before = service.version();
            service
                .start(1, at("2026-08-28T15:43:21-07:00"))
                .await
                .expect("ok");
            let after_start = service.version();
            service.finish(1).await.expect("ok");

            assert!(after_start > before);
            assert!(service.version() > after_start);
        });
    }

    #[test]
    fn the_key_is_removed_rather_than_left_empty() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);

            service
                .start(1, at("2026-08-28T15:43:21-07:00"))
                .await
                .expect("ok");
            assert!(store.contains(STORAGE_KEY));

            service.finish(1).await.expect("ok");
            assert!(
                !store.contains(STORAGE_KEY),
                "the key must be removed, not written as []"
            );
        });
    }

    #[test]
    fn persisted_state_matches_the_recorded_csharp_payload() {
        block_on(async {
            let fixture = read_json_fixture("active-activities.json");
            let expected = fixture["afterThreeStarts"].as_str().expect("payload");

            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);
            for (id, when) in [
                (1, "2026-08-28T15:43:21-07:00"),
                (7, "2026-06-15T10:00:00Z"),
                (42, "2026-01-01T00:00:00"),
                (99, "2026-08-28T15:43:21.1234567-07:00"),
                (100, "2026-08-28T15:43:21.1-07:00"),
            ] {
                service.start(id, at(when)).await.expect("ok");
            }

            let written = store.snapshot();
            assert_eq!(
                written.get(STORAGE_KEY).map(String::as_str),
                Some(expected),
                "the persisted payload must match what the C# wrote"
            );
        });
    }

    #[test]
    fn state_survives_a_reload() {
        block_on(async {
            let store = MemStorage::new();
            {
                let service = ActiveActivityService::new(&store);
                service
                    .start(1, at("2026-08-28T15:43:21.1234567-07:00"))
                    .await
                    .expect("ok");
            }

            let reloaded = ActiveActivityService::new(&store);
            reloaded.initialize().await.expect("ok");

            assert!(reloaded.is_active(1));
            assert_eq!(
                reloaded.all().get(&1).map(|t| t.to_wire()),
                Some("2026-08-28T15:43:21.1234567-07:00".to_owned())
            );
        });
    }

    #[test]
    fn corrupt_state_is_discarded_silently() {
        block_on(async {
            let store = MemStorage::seeded([(STORAGE_KEY, "{not json")]);
            let service = ActiveActivityService::new(&store);

            service
                .initialize()
                .await
                .expect("initialize must not fail");

            assert!(service.is_empty());
            assert!(!store.contains(STORAGE_KEY), "corrupt state is cleared");
        });
    }

    #[test]
    fn an_absent_key_initializes_to_empty() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActiveActivityService::new(&store);
            service.initialize().await.expect("ok");
            assert!(service.is_empty());
        });
    }
}
