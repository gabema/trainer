//! `localStorage`-backed [`Storage`], replacing the `localStorage.*` interop
//! calls in `LocalStorageService` and `ActiveActivityService`.
//!
//! Also serves as the source side of the legacy migration: profiles predating
//! the IndexedDB switch keep their `activities` and `activityTypes` here.
//!
//! Unlike IndexedDB, localStorage stores strings natively, so no parse or
//! stringify happens at this boundary.

use async_trait::async_trait;
use trainer_core::storage::{Storage, StorageError, StorageResult};

fn storage() -> StorageResult<web_sys::Storage> {
    web_sys::window()
        .ok_or_else(|| StorageError::new("local_storage", "", "no window"))?
        .local_storage()
        .map_err(|e| StorageError::new("local_storage", "", format!("{e:?}")))?
        .ok_or_else(|| StorageError::new("local_storage", "", "localStorage unavailable"))
}

/// Browser `localStorage`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalStorage;

impl LocalStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl Storage for LocalStorage {
    async fn get_item(&self, key: &str) -> StorageResult<Option<String>> {
        storage()?
            .get_item(key)
            .map_err(|e| StorageError::new("get_item", key, format!("{e:?}")))
    }

    async fn set_item(&self, key: &str, value: &str) -> StorageResult<()> {
        storage()?
            .set_item(key, value)
            .map_err(|e| StorageError::new("set_item", key, format!("{e:?}")))
    }

    async fn remove_item(&self, key: &str) -> StorageResult<()> {
        storage()?
            .remove_item(key)
            .map_err(|e| StorageError::new("remove_item", key, format!("{e:?}")))
    }

    async fn clear(&self) -> StorageResult<()> {
        storage()?
            .clear()
            .map_err(|e| StorageError::new("clear", "", format!("{e:?}")))
    }

    async fn keys_with_prefix(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let store = storage()?;
        let length = store
            .length()
            .map_err(|e| StorageError::new("length", prefix, format!("{e:?}")))?;

        let mut matching = Vec::new();
        for index in 0..length {
            let key = store
                .key(index)
                .map_err(|e| StorageError::new("key", prefix, format!("{e:?}")))?;
            if let Some(key) = key
                && key.starts_with(prefix)
            {
                matching.push(key);
            }
        }
        Ok(matching)
    }
}
