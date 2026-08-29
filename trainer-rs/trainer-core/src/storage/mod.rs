//! The storage seam, ported from `Trainer/Services/IStorageService.cs`.
//!
//! # Why the trait deals in strings
//!
//! The C# interface is generic — `GetItemAsync<T>(string key)` — which is not
//! object-safe in Rust. It is also only ever sugar: the implementation
//! serializes to JSON and hands a string across the JS boundary. This trait
//! therefore exposes the string operations directly and leaves serialization to
//! callers, which matches what actually crosses the boundary.
//!
//! # Why `?Send`
//!
//! WASM futures are not `Send`, so every method is declared `?Send` and that
//! bound propagates through every service. Deciding it here rather than at the
//! fifteenth trait avoids a miserable retrofit.
//!
//! # Layers
//!
//! ```text
//! services  ->  WeekBucketed<S>  ->  MemStorage   (native tests)
//!                                    IdbStorage   (browser)
//!                                    LocalStorage (browser, legacy source)
//! ```
//!
//! [`WeekBucketed`] is a decorator that itself implements [`Storage`], so it
//! reproduces `IndexedDbStorageService`'s special handling of the `activities`
//! key while remaining testable without a browser.

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::fmt;

pub mod buckets;
pub mod mem;
pub mod migration;

pub use buckets::WeekBucketed;
pub use mem::MemStorage;

/// A storage operation that failed at the boundary.
///
/// The C# catches `JSException` and returns a default in most read paths; that
/// recovery is applied by callers here rather than being baked into the trait,
/// so a genuine failure stays visible to code that cares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageError {
    pub operation: &'static str,
    pub key: String,
    pub detail: String,
}

impl StorageError {
    pub fn new(operation: &'static str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            operation,
            key: key.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "storage {} failed for key {:?}: {}",
            self.operation, self.key, self.detail
        )
    }
}

impl std::error::Error for StorageError {}

pub type StorageResult<T> = Result<T, StorageError>;

/// Key/value storage holding JSON strings.
///
/// Mirrors `IStorageService` plus the two batch operations the JavaScript shim
/// exposes and `IndexedDbStorageService` depends on.
#[async_trait(?Send)]
pub trait Storage {
    async fn get_item(&self, key: &str) -> StorageResult<Option<String>>;
    async fn set_item(&self, key: &str, value: &str) -> StorageResult<()>;
    async fn remove_item(&self, key: &str) -> StorageResult<()>;
    async fn clear(&self) -> StorageResult<()>;

    /// Every key beginning with `prefix`. Mirrors `getAllKeysWithPrefix`.
    async fn keys_with_prefix(&self, prefix: &str) -> StorageResult<Vec<String>>;

    /// Fetches many keys at once, omitting any that are absent. Mirrors
    /// `getItems`, whose result the C# deserializes into a dictionary.
    async fn get_items(&self, keys: &[String]) -> StorageResult<BTreeMap<String, String>> {
        let mut found = BTreeMap::new();
        for key in keys {
            if let Some(value) = self.get_item(key).await? {
                found.insert(key.clone(), value);
            }
        }
        Ok(found)
    }
}
