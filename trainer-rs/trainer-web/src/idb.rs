//! IndexedDB storage, replacing `wwwroot/js/indexeddb-storage.js`.
//!
//! # Value representation
//!
//! The JavaScript shim `JSON.parse`s on write and `JSON.stringify`s on read, so
//! IndexedDB holds **structured-cloned values**, not strings. A real profile
//! dump confirms it: 33 entries are `Array`, and `activityNextId` is a bare
//! `Number`. Storing JSON strings directly — the obvious simplification — would
//! make every existing user's data unreadable, so that boundary is preserved
//! here with `js_sys::JSON`.
//!
//! # Database version
//!
//! Opened at version 1 with an upgrade handler that creates the object store,
//! exactly as the shim does. Opening at version 1 against an existing version-1
//! database does not fire `onupgradeneeded`, so existing profiles are untouched;
//! opening *without* a version would leave a fresh profile with no object store
//! at all.
//!
//! # Closures
//!
//! Every IndexedDB operation is an event-driven `IdbRequest`. [`request_future`]
//! wraps one in a `Promise` so it can be awaited, and is the only place in the
//! port that manages `Closure` lifetimes.

use async_trait::async_trait;
use js_sys::{Array, JSON};
use trainer_core::storage::{Storage, StorageError, StorageResult};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbObjectStore, IdbRequest, IdbTransaction, IdbTransactionMode};

/// Database and object store names, fixed by the existing data.
pub const DEFAULT_DB_NAME: &str = "Trainer";
pub const DB_VERSION: u32 = 1;
pub const STORE_NAME: &str = "activities";

fn err(
    operation: &'static str,
    key: impl Into<String>,
    detail: impl std::fmt::Debug,
) -> StorageError {
    StorageError::new(operation, key, format!("{detail:?}"))
}

/// Awaits an `IdbRequest`, resolving with its result or rejecting with its error.
///
/// The handlers are installed via `Closure::once_into_js`, which frees the
/// closure after it fires. Exactly one of the two fires, so the other leaks a
/// small allocation. That is the accepted cost of not dropping a `Closure`
/// while its own JS callback is still on the stack, which would be unsound.
pub async fn await_request(request: IdbRequest) -> StorageResult<JsValue> {
    request_future(request, "request").await
}

async fn request_future(request: IdbRequest, operation: &'static str) -> StorageResult<JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let success_request = request.clone();
        let on_success = Closure::once_into_js(move |_event: web_sys::Event| {
            let value = success_request.result().unwrap_or(JsValue::UNDEFINED);
            let _ = resolve.call1(&JsValue::NULL, &value);
        });

        let error_request = request.clone();
        let on_error = Closure::once_into_js(move |_event: web_sys::Event| {
            let reason = error_request
                .error()
                .ok()
                .flatten()
                .map(JsValue::from)
                .unwrap_or_else(|| JsValue::from_str("IndexedDB request failed"));
            let _ = reject.call1(&JsValue::NULL, &reason);
        });

        request.set_onsuccess(on_success.dyn_ref());
        request.set_onerror(on_error.dyn_ref());
    });

    JsFuture::from(promise)
        .await
        .map_err(|e| err(operation, "", e))
}

/// IndexedDB-backed [`Storage`].
#[derive(Debug, Clone)]
pub struct IdbStorage {
    db_name: String,
}

impl Default for IdbStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl IdbStorage {
    /// Opens the app's database.
    pub fn new() -> Self {
        Self {
            db_name: DEFAULT_DB_NAME.to_owned(),
        }
    }

    /// Opens a differently named database. Tests use this for isolation; the
    /// app always uses [`DEFAULT_DB_NAME`].
    pub fn with_database(name: impl Into<String>) -> Self {
        Self {
            db_name: name.into(),
        }
    }

    async fn open(&self) -> StorageResult<IdbDatabase> {
        let factory = web_sys::window()
            .ok_or_else(|| StorageError::new("open", &self.db_name, "no window"))?
            .indexed_db()
            .map_err(|e| err("open", &self.db_name, e))?
            .ok_or_else(|| StorageError::new("open", &self.db_name, "indexedDB unavailable"))?;

        let request = factory
            .open_with_u32(&self.db_name, DB_VERSION)
            .map_err(|e| err("open", &self.db_name, e))?;

        // Creates the object store on a fresh profile. Does not fire for an
        // existing version-1 database.
        let upgrade_request = request.clone();
        let on_upgrade = Closure::once_into_js(move |_event: web_sys::Event| {
            if let Ok(result) = upgrade_request.result()
                && let Ok(db) = result.dyn_into::<IdbDatabase>()
                && !db.object_store_names().contains(STORE_NAME)
            {
                let _ = db.create_object_store(STORE_NAME);
            }
        });
        request.set_onupgradeneeded(on_upgrade.dyn_ref());

        let value = request_future(request.unchecked_into(), "open").await?;
        value
            .dyn_into::<IdbDatabase>()
            .map_err(|e| err("open", &self.db_name, e))
    }

    async fn store(
        &self,
        mode: IdbTransactionMode,
    ) -> StorageResult<(IdbDatabase, IdbObjectStore)> {
        let db = self.open().await?;
        let transaction: IdbTransaction = db
            .transaction_with_str_and_mode(STORE_NAME, mode)
            .map_err(|e| err("transaction", STORE_NAME, e))?;
        let store = transaction
            .object_store(STORE_NAME)
            .map_err(|e| err("object_store", STORE_NAME, e))?;
        Ok((db, store))
    }
}

#[async_trait(?Send)]
impl Storage for IdbStorage {
    async fn get_item(&self, key: &str) -> StorageResult<Option<String>> {
        let (db, store) = self.store(IdbTransactionMode::Readonly).await?;
        let request = store
            .get(&JsValue::from_str(key))
            .map_err(|e| err("get_item", key, e))?;
        let value = request_future(request, "get_item").await?;
        db.close();

        if value.is_undefined() || value.is_null() {
            return Ok(None);
        }

        // Stringify at the boundary, mirroring the shim, so callers see JSON.
        JSON::stringify(&value)
            .map(|s| Some(String::from(s)))
            .map_err(|e| err("get_item", key, e))
    }

    async fn set_item(&self, key: &str, value: &str) -> StorageResult<()> {
        // Parse before writing, so IndexedDB holds a structured-cloned value
        // rather than a string. This is the compatibility-critical step.
        let parsed = JSON::parse(value).map_err(|e| err("set_item", key, e))?;

        let (db, store) = self.store(IdbTransactionMode::Readwrite).await?;
        let request = store
            .put_with_key(&parsed, &JsValue::from_str(key))
            .map_err(|e| err("set_item", key, e))?;
        request_future(request, "set_item").await?;
        db.close();
        Ok(())
    }

    async fn remove_item(&self, key: &str) -> StorageResult<()> {
        let (db, store) = self.store(IdbTransactionMode::Readwrite).await?;
        let request = store
            .delete(&JsValue::from_str(key))
            .map_err(|e| err("remove_item", key, e))?;
        request_future(request, "remove_item").await?;
        db.close();
        Ok(())
    }

    async fn clear(&self) -> StorageResult<()> {
        let (db, store) = self.store(IdbTransactionMode::Readwrite).await?;
        let request = store.clear().map_err(|e| err("clear", "", e))?;
        request_future(request, "clear").await?;
        db.close();
        Ok(())
    }

    async fn keys_with_prefix(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let (db, store) = self.store(IdbTransactionMode::Readonly).await?;
        let request = store
            .get_all_keys()
            .map_err(|e| err("keys_with_prefix", prefix, e))?;
        let value = request_future(request, "keys_with_prefix").await?;
        db.close();

        let keys = Array::from(&value);
        let mut matching = Vec::new();
        for key in keys.iter() {
            if let Some(text) = key.as_string()
                && text.starts_with(prefix)
            {
                matching.push(text);
            }
        }
        Ok(matching)
    }
}
