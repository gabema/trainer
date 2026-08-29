//! Activity types, ported from `Trainer/Services/ActivityTypeService.cs`.

use crate::models::{self, ActivityType, Format};
use crate::storage::{Storage, StorageError, StorageResult};

const STORAGE_KEY: &str = "activityTypes";

/// CRUD over the stored activity type list.
///
/// Ids are assigned as `max(existing) + 1`, recomputed from storage on each
/// read, matching the C#. Reads are sorted by name while writes preserve
/// storage order, which is why the two paths are separate.
pub struct ActivityTypeService<'a, S: Storage> {
    storage: &'a S,
}

impl<'a, S: Storage> ActivityTypeService<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        Self { storage }
    }

    /// Storage order, untouched. Writes operate on this so the stored order is
    /// preserved across edits.
    async fn all_unsorted(&self) -> StorageResult<Vec<ActivityType>> {
        match self.storage.get_item(STORAGE_KEY).await? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| StorageError::new("deserialize", STORAGE_KEY, e.to_string())),
            None => Ok(Vec::new()),
        }
    }

    fn next_id(types: &[ActivityType]) -> i32 {
        types.iter().map(|t| t.id).max().map_or(1, |max| max + 1)
    }

    async fn save(&self, types: &Vec<ActivityType>) -> StorageResult<()> {
        let json = models::to_json(types, Format::Storage)
            .map_err(|e| StorageError::new("serialize", STORAGE_KEY, e.to_string()))?;
        self.storage.set_item(STORAGE_KEY, &json).await
    }

    /// Every activity type, sorted by name for display.
    pub async fn all(&self) -> StorageResult<Vec<ActivityType>> {
        let mut types = self.all_unsorted().await?;
        types.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(types)
    }

    pub async fn by_id(&self, id: i32) -> StorageResult<Option<ActivityType>> {
        Ok(self.all().await?.into_iter().find(|t| t.id == id))
    }

    /// Appends a type, assigning it the next id.
    pub async fn add(&self, mut activity_type: ActivityType) -> StorageResult<ActivityType> {
        let mut types = self.all_unsorted().await?;
        activity_type.id = Self::next_id(&types);
        types.push(activity_type.clone());
        self.save(&types).await?;
        Ok(activity_type)
    }

    /// Replaces a type in place. A type that is not present is ignored, as the
    /// C# `FindIndex` guard does.
    pub async fn update(&self, activity_type: ActivityType) -> StorageResult<()> {
        let mut types = self.all_unsorted().await?;
        if let Some(slot) = types.iter_mut().find(|t| t.id == activity_type.id) {
            *slot = activity_type;
            self.save(&types).await?;
        }
        Ok(())
    }

    pub async fn delete(&self, id: i32) -> StorageResult<()> {
        let mut types = self.all_unsorted().await?;
        types.retain(|t| t.id != id);
        self.save(&types).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::block_on;
    use crate::models::NetBenefit;
    use crate::storage::MemStorage;

    fn make(id: i32, name: &str) -> ActivityType {
        ActivityType {
            id,
            name: name.to_owned(),
            net_benefit: NetBenefit::Neutral,
            daily_amount: None,
            weekly_amount: None,
            unit: None,
            is_private: false,
            decimal_places: 0,
        }
    }

    fn seeded(types: &[ActivityType]) -> MemStorage {
        let json = models::to_json(&types.to_vec(), Format::Storage).expect("serializes");
        MemStorage::seeded([(STORAGE_KEY, json.as_str())])
    }

    /// Ports `GetAllAsync_ReturnsEmptyList_WhenNoTypesExist`.
    #[test]
    fn returns_empty_when_nothing_is_stored() {
        block_on(async {
            let store = MemStorage::new();
            assert!(
                ActivityTypeService::new(&store)
                    .all()
                    .await
                    .expect("ok")
                    .is_empty()
            );
        });
    }

    /// Ports `GetAllAsync_ReturnsActivityTypesSortedByName`.
    #[test]
    fn reads_are_sorted_by_name() {
        block_on(async {
            let store = seeded(&[make(1, "Swimming"), make(2, "Cycling"), make(3, "Running")]);
            let names: Vec<String> = ActivityTypeService::new(&store)
                .all()
                .await
                .expect("ok")
                .into_iter()
                .map(|t| t.name)
                .collect();
            assert_eq!(names, vec!["Cycling", "Running", "Swimming"]);
        });
    }

    /// Ports `WriteOperations_PreserveStorageOrder_WhileGetAllAsyncReturnsSorted`.
    #[test]
    fn writes_preserve_storage_order_while_reads_sort() {
        block_on(async {
            let store = seeded(&[make(1, "Swimming"), make(2, "Cycling")]);
            let service = ActivityTypeService::new(&store);

            service.add(make(0, "Archery")).await.expect("ok");

            // Sorted for display...
            let sorted: Vec<String> = service
                .all()
                .await
                .expect("ok")
                .into_iter()
                .map(|t| t.name)
                .collect();
            assert_eq!(sorted, vec!["Archery", "Cycling", "Swimming"]);

            // ...but appended in storage order, not re-sorted on disk.
            let raw = store.snapshot();
            let stored: Vec<ActivityType> =
                serde_json::from_str(&raw[STORAGE_KEY]).expect("parses");
            let stored_names: Vec<&str> = stored.iter().map(|t| t.name.as_str()).collect();
            assert_eq!(stored_names, vec!["Swimming", "Cycling", "Archery"]);
        });
    }

    /// Ports `GetByIdAsync_ReturnsActivityType_WhenExists` / `_WhenNotExists`.
    #[test]
    fn finds_by_id_or_returns_none() {
        block_on(async {
            let store = seeded(&[make(1, "Swimming"), make(2, "Cycling")]);
            let service = ActivityTypeService::new(&store);
            assert_eq!(
                service.by_id(2).await.expect("ok").map(|t| t.name),
                Some("Cycling".to_owned())
            );
            assert!(service.by_id(99).await.expect("ok").is_none());
        });
    }

    /// Ports `AddAsync_AddsActivityTypeWithNewId`.
    #[test]
    fn add_assigns_the_next_id() {
        block_on(async {
            let store = MemStorage::new();
            let service = ActivityTypeService::new(&store);

            let first = service.add(make(0, "A")).await.expect("ok");
            let second = service.add(make(0, "B")).await.expect("ok");

            assert_eq!(first.id, 1, "an empty store starts at one");
            assert_eq!(second.id, 2);
        });
    }

    #[test]
    fn add_continues_from_the_highest_existing_id() {
        block_on(async {
            let store = seeded(&[make(7, "Existing")]);
            let added = ActivityTypeService::new(&store)
                .add(make(0, "New"))
                .await
                .expect("ok");
            assert_eq!(added.id, 8);
        });
    }

    /// Ports `UpdateAsync_UpdatesExistingActivityType`.
    #[test]
    fn update_replaces_in_place() {
        block_on(async {
            let store = seeded(&[make(1, "Old"), make(2, "Other")]);
            let service = ActivityTypeService::new(&store);

            let mut edited = make(1, "New");
            edited.decimal_places = 2;
            service.update(edited).await.expect("ok");

            let found = service.by_id(1).await.expect("ok").expect("present");
            assert_eq!(found.name, "New");
            assert_eq!(found.decimal_places, 2);
            assert_eq!(service.all().await.expect("ok").len(), 2);
        });
    }

    #[test]
    fn updating_an_absent_type_is_ignored() {
        block_on(async {
            let store = seeded(&[make(1, "Only")]);
            let service = ActivityTypeService::new(&store);
            service.update(make(99, "Ghost")).await.expect("ok");
            assert_eq!(service.all().await.expect("ok").len(), 1);
        });
    }

    /// Ports `DeleteAsync_RemovesActivityType`.
    #[test]
    fn delete_removes_by_id() {
        block_on(async {
            let store = seeded(&[make(1, "A"), make(2, "B")]);
            let service = ActivityTypeService::new(&store);

            service.delete(1).await.expect("ok");
            let remaining = service.all().await.expect("ok");
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining[0].id, 2);
        });
    }

    #[test]
    fn reads_the_real_profile_types() {
        block_on(async {
            let snapshot = crate::fixtures::read_json_fixture("idb-snapshot.json");
            let json =
                serde_json::to_string(&snapshot["entries"]["activityTypes"]["value"]).expect("ok");
            let store = MemStorage::seeded([(STORAGE_KEY, json.as_str())]);

            let types = ActivityTypeService::new(&store).all().await.expect("ok");
            assert_eq!(types.len(), 16);
            // Sorted for display.
            let mut names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
            let sorted = {
                let mut c = names.clone();
                c.sort_unstable();
                c
            };
            names.dedup();
            assert_eq!(
                types.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
                sorted
            );
        });
    }
}
