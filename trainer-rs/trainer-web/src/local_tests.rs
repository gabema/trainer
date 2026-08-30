//! Browser-tier tests for [`crate::local::LocalStorage`].
//!
//! Ports `Trainer.Tests/Services/LocalStorageServiceTests.cs`, which ran against
//! an `InMemoryJsRuntime` stub and so proved only that the C# wrapper called the
//! right interop names. These run against the browser's real `localStorage`,
//! which is where the active-activity set actually lives.
//!
//! Keys are prefixed per test rather than isolated per store the way
//! `idb_tests` opens a unique database: localStorage has one namespace per
//! origin and no equivalent of a fresh database.

use crate::local::LocalStorage;
use std::sync::atomic::{AtomicU32, Ordering};
use trainer_core::services::active_activity::ActiveActivityService;
use trainer_core::services::active_time::ActiveTime;
use trainer_core::storage::Storage;
use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

/// A key prefix no other test shares.
fn unique_prefix(label: &str) -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    format!(
        "__test_{label}_{}_",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

fn store() -> LocalStorage {
    LocalStorage::new()
}

#[wasm_bindgen_test]
async fn a_value_round_trips() {
    let prefix = unique_prefix("round_trip");
    let key = format!("{prefix}value");
    store().set_item(&key, "stored").await.expect("set");

    assert_eq!(
        store().get_item(&key).await.expect("get"),
        Some("stored".to_owned())
    );

    store().remove_item(&key).await.expect("remove");
}

/// Ports `GetItemAsync_ReturnsDefault_WhenItemDoesNotExist`. The C# returned a
/// default; the trait returns `None`, which is the distinction the migration
/// path depends on — an absent key and an empty string are not the same thing.
#[wasm_bindgen_test]
async fn a_missing_key_reads_as_none() {
    let key = format!("{}absent", unique_prefix("missing"));
    assert_eq!(store().get_item(&key).await.expect("get"), None);
}

#[wasm_bindgen_test]
async fn an_empty_string_is_kept_distinct_from_absence() {
    let key = format!("{}empty", unique_prefix("empty"));
    store().set_item(&key, "").await.expect("set");

    assert_eq!(
        store().get_item(&key).await.expect("get"),
        Some(String::new()),
        "an empty stored value must not read back as a missing key"
    );

    store().remove_item(&key).await.expect("remove");
}

#[wasm_bindgen_test]
async fn setting_an_existing_key_overwrites_it() {
    let key = format!("{}overwrite", unique_prefix("overwrite"));
    store().set_item(&key, "first").await.expect("set");
    store().set_item(&key, "second").await.expect("set again");

    assert_eq!(
        store().get_item(&key).await.expect("get"),
        Some("second".to_owned())
    );

    store().remove_item(&key).await.expect("remove");
}

#[wasm_bindgen_test]
async fn remove_deletes_only_the_named_key() {
    let prefix = unique_prefix("remove");
    let doomed = format!("{prefix}doomed");
    let kept = format!("{prefix}kept");
    store().set_item(&doomed, "a").await.expect("set");
    store().set_item(&kept, "b").await.expect("set");

    store().remove_item(&doomed).await.expect("remove");

    assert_eq!(store().get_item(&doomed).await.expect("get"), None);
    assert_eq!(
        store().get_item(&kept).await.expect("get"),
        Some("b".to_owned()),
        "removing one key must not disturb its neighbours"
    );

    store().remove_item(&kept).await.expect("remove");
}

#[wasm_bindgen_test]
async fn prefix_search_matches_only_the_prefix() {
    let prefix = unique_prefix("prefix");
    let inside_one = format!("{prefix}2026.01");
    let inside_two = format!("{prefix}2026.02");
    let outside = format!("other_{prefix}2026.03");
    for key in [&inside_one, &inside_two, &outside] {
        store().set_item(key, "x").await.expect("set");
    }

    let mut found = store()
        .keys_with_prefix(&prefix)
        .await
        .expect("prefix scan");
    found.sort();

    assert_eq!(found, vec![inside_one.clone(), inside_two.clone()]);

    for key in [&inside_one, &inside_two, &outside] {
        store().remove_item(key).await.expect("remove");
    }
}

/// `clear` is deliberately not exercised against the live store: it empties the
/// whole origin, including whatever the surrounding test page is holding. What
/// matters for the app is that the active-activity set survives a write and a
/// read through the real store, which is the only thing localStorage is used
/// for once the legacy migration has run.
#[wasm_bindgen_test]
async fn the_active_activity_set_persists_through_the_real_store() {
    let storage = LocalStorage::new();
    let service = ActiveActivityService::new(&storage);
    service.initialize().await.expect("initialize");

    let start = ActiveTime::Unspecified(
        chrono::NaiveDate::from_ymd_opt(2026, 2, 3)
            .expect("valid date")
            .and_hms_opt(7, 15, 0)
            .expect("valid time"),
    );
    service.start(4242, start).await.expect("start");

    // A second service over the same store, standing in for a page reload.
    let reloaded = ActiveActivityService::new(&storage);
    reloaded.initialize().await.expect("re-initialize");
    assert!(reloaded.is_active(4242), "the timer survived the reload");
    assert_eq!(reloaded.all().get(&4242).copied(), Some(start));

    reloaded.finish(4242).await.expect("finish");

    let after = ActiveActivityService::new(&storage);
    after.initialize().await.expect("re-initialize");
    assert!(!after.is_active(4242), "finishing is persisted too");
}
