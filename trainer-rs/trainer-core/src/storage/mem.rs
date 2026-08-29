//! In-memory [`Storage`], standing in for the Moq mocks the C# tests use.
//!
//! Keeps the service tests in the fast native tier: everything above the
//! storage boundary can be exercised without a browser.

use super::{Storage, StorageResult};
use async_trait::async_trait;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// A `BTreeMap`-backed store. Not `Sync`, matching the single-threaded browser
/// environment the real implementations run in.
#[derive(Debug, Default)]
pub struct MemStorage {
    items: RefCell<BTreeMap<String, String>>,
}

impl MemStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds the store, for tests that start from an existing profile.
    pub fn seeded<I, K, V>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let store = Self::new();
        {
            let mut items = store.items.borrow_mut();
            for (key, value) in entries {
                items.insert(key.into(), value.into());
            }
        }
        store
    }

    /// Snapshot of everything held, for assertions.
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.items.borrow().clone()
    }

    pub fn len(&self) -> usize {
        self.items.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.borrow().is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.items.borrow().contains_key(key)
    }
}

#[async_trait(?Send)]
impl Storage for MemStorage {
    async fn get_item(&self, key: &str) -> StorageResult<Option<String>> {
        Ok(self.items.borrow().get(key).cloned())
    }

    async fn set_item(&self, key: &str, value: &str) -> StorageResult<()> {
        self.items
            .borrow_mut()
            .insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    async fn remove_item(&self, key: &str) -> StorageResult<()> {
        self.items.borrow_mut().remove(key);
        Ok(())
    }

    async fn clear(&self) -> StorageResult<()> {
        self.items.borrow_mut().clear();
        Ok(())
    }

    async fn keys_with_prefix(&self, prefix: &str) -> StorageResult<Vec<String>> {
        Ok(self
            .items
            .borrow()
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::block_on;

    /// Ports `LocalStorageServiceTests` — the same operations, against the
    /// in-memory implementation rather than a mocked JS runtime.
    #[test]
    fn round_trips_values() {
        block_on(async {
            let store = MemStorage::new();
            assert_eq!(store.get_item("missing").await.expect("ok"), None);

            store.set_item("k", "\"value\"").await.expect("ok");
            assert_eq!(
                store.get_item("k").await.expect("ok"),
                Some("\"value\"".to_owned())
            );
        });
    }

    #[test]
    fn overwrites_an_existing_value() {
        block_on(async {
            let store = MemStorage::new();
            store.set_item("k", "1").await.expect("ok");
            store.set_item("k", "2").await.expect("ok");
            assert_eq!(store.get_item("k").await.expect("ok"), Some("2".to_owned()));
        });
    }

    #[test]
    fn removes_and_clears() {
        block_on(async {
            let store = MemStorage::seeded([("a", "1"), ("b", "2")]);
            store.remove_item("a").await.expect("ok");
            assert_eq!(store.get_item("a").await.expect("ok"), None);
            assert_eq!(store.get_item("b").await.expect("ok"), Some("2".to_owned()));

            store.clear().await.expect("ok");
            assert!(store.is_empty());
        });
    }

    #[test]
    fn removing_a_missing_key_is_not_an_error() {
        block_on(async {
            let store = MemStorage::new();
            store.remove_item("nope").await.expect("ok");
        });
    }

    #[test]
    fn filters_keys_by_prefix() {
        block_on(async {
            let store = MemStorage::seeded([
                ("activities-2026.01", "[]"),
                ("activities-2026.02", "[]"),
                ("activityTypes", "[]"),
                ("activityNextId", "1"),
            ]);

            let mut keys = store.keys_with_prefix("activities-").await.expect("ok");
            keys.sort();
            assert_eq!(keys, vec!["activities-2026.01", "activities-2026.02"]);

            // "activityTypes" and "activityNextId" share the prefix "activit"
            // but not "activities-", which is the distinction that matters.
            assert!(!keys.iter().any(|k| k == "activityTypes"));
        });
    }

    #[test]
    fn batch_get_omits_absent_keys() {
        block_on(async {
            let store = MemStorage::seeded([("a", "1"), ("c", "3")]);
            let found = store
                .get_items(&["a".to_owned(), "b".to_owned(), "c".to_owned()])
                .await
                .expect("ok");

            assert_eq!(found.len(), 2);
            assert_eq!(found.get("a").map(String::as_str), Some("1"));
            assert_eq!(found.get("c").map(String::as_str), Some("3"));
            assert!(!found.contains_key("b"));
        });
    }
}
