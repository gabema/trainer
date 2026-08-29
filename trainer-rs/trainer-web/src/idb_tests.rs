//! Browser-tier verification of [`crate::idb::IdbStorage`] against the real
//! de-identified profile dump. Task 7.7.
//!
//! Each test uses its own database name so runs cannot interfere with one
//! another or with a developer's real `Trainer` database.

use crate::idb::{IdbStorage, STORE_NAME};
use js_sys::{Array, JSON};
use trainer_core::fixtures::embedded::IDB_SNAPSHOT;
use trainer_core::storage::{Storage, WeekBucketed};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

fn unique_db(label: &str) -> String {
    let stamp = js_sys::Date::now() as u64;
    format!("TrainerTest-{label}-{stamp}")
}

fn snapshot() -> serde_json::Value {
    serde_json::from_str(IDB_SNAPSHOT).expect("the embedded snapshot parses")
}

/// Seeds a database with every entry from the real profile dump.
async fn seed(storage: &IdbStorage) -> serde_json::Value {
    let snapshot = snapshot();
    let entries = snapshot["entries"].as_object().expect("entries").clone();
    for (key, entry) in &entries {
        let json = serde_json::to_string(&entry["value"]).expect("serializes");
        storage.set_item(key, &json).await.expect("seeded");
    }
    snapshot
}

/// Reads a raw stored value back through the JS API, bypassing `IdbStorage`,
/// so the on-disk representation can be inspected rather than assumed.
async fn raw_value(db_name: &str, key: &str) -> JsValue {
    let factory = web_sys::window()
        .expect("window")
        .indexed_db()
        .expect("indexedDB accessible")
        .expect("indexedDB present");
    let open = factory.open_with_u32(db_name, 1).expect("open");
    let db_value = crate::idb::await_request(open.clone().unchecked_into())
        .await
        .expect("opened");
    let db: web_sys::IdbDatabase = db_value.dyn_into().expect("database");
    let store = db
        .transaction_with_str(STORE_NAME)
        .expect("transaction")
        .object_store(STORE_NAME)
        .expect("store");
    let request = store.get(&JsValue::from_str(key)).expect("get");
    let value = crate::idb::await_request(request).await.expect("value");
    db.close();
    value
}

#[wasm_bindgen_test]
async fn values_are_stored_as_structured_clones_not_strings() {
    let name = unique_db("repr");
    let storage = IdbStorage::with_database(&name);
    seed(&storage).await;

    // The compatibility-critical assertion: a week bucket must come back as a
    // JS Array. Storing JSON strings instead would orphan every user's data.
    let bucket = raw_value(&name, "activities-2026.01").await;
    assert!(
        Array::is_array(&bucket),
        "week buckets must be structured-cloned arrays, got {:?}",
        JSON::stringify(&bucket).map(String::from)
    );
    assert!(!bucket.is_string());

    // ...and the scalar key must stay a bare number.
    let next_id = raw_value(&name, "activityNextId").await;
    assert!(
        next_id.as_f64().is_some(),
        "activityNextId must be a number, got {:?}",
        JSON::stringify(&next_id).map(String::from)
    );
    assert!(!next_id.is_string());
}

#[wasm_bindgen_test]
async fn every_key_shape_from_the_real_profile_round_trips() {
    let name = unique_db("shapes");
    let storage = IdbStorage::with_database(&name);
    let snapshot = seed(&storage).await;
    let entries = snapshot["entries"].as_object().expect("entries");

    // Task 7.4a: Array buckets, the two unbucketed Arrays, and the Number.
    for key in [
        "activities-2026.01",
        "activityTypes",
        "knownLocations",
        "activityNextId",
    ] {
        let stored = storage
            .get_item(key)
            .await
            .expect("read succeeds")
            .unwrap_or_else(|| panic!("{key} should be present"));

        let expected = serde_json::to_string(&entries[key]["value"]).expect("serializes");
        let stored_value: serde_json::Value =
            serde_json::from_str(&stored).expect("stored value parses");
        let expected_value: serde_json::Value =
            serde_json::from_str(&expected).expect("expected value parses");

        assert_eq!(stored_value, expected_value, "{key} did not round-trip");
    }
}

#[wasm_bindgen_test]
async fn missing_keys_read_as_none() {
    let storage = IdbStorage::with_database(unique_db("missing"));
    assert_eq!(storage.get_item("nothing-here").await.expect("ok"), None);
}

#[wasm_bindgen_test]
async fn remove_and_clear_behave() {
    let name = unique_db("remove");
    let storage = IdbStorage::with_database(&name);
    seed(&storage).await;

    storage.remove_item("activityNextId").await.expect("ok");
    assert_eq!(storage.get_item("activityNextId").await.expect("ok"), None);
    assert!(
        storage
            .get_item("activityTypes")
            .await
            .expect("ok")
            .is_some()
    );

    storage.clear().await.expect("ok");
    assert!(storage.keys_with_prefix("").await.expect("ok").is_empty());
}

#[wasm_bindgen_test]
async fn prefix_search_matches_the_shim() {
    let name = unique_db("prefix");
    let storage = IdbStorage::with_database(&name);
    seed(&storage).await;

    let keys = storage.keys_with_prefix("activities-").await.expect("ok");
    assert_eq!(keys.len(), 31, "31 week buckets in the real profile");
    assert!(keys.iter().all(|k| k.starts_with("activities-")));
    // Sibling keys sharing the "activit" prefix must be excluded.
    assert!(!keys.iter().any(|k| k == "activityTypes"));
    assert!(!keys.iter().any(|k| k == "activityNextId"));
}

#[wasm_bindgen_test]
async fn batch_get_omits_absent_keys() {
    let name = unique_db("batch");
    let storage = IdbStorage::with_database(&name);
    seed(&storage).await;

    let found = storage
        .get_items(&[
            "activityTypes".to_owned(),
            "does-not-exist".to_owned(),
            "knownLocations".to_owned(),
        ])
        .await
        .expect("ok");

    assert_eq!(found.len(), 2);
    assert!(found.contains_key("activityTypes"));
    assert!(!found.contains_key("does-not-exist"));
}

#[wasm_bindgen_test]
async fn opening_an_existing_database_does_not_upgrade_it() {
    let name = unique_db("noupgrade");
    let storage = IdbStorage::with_database(&name);
    storage.set_item("activityNextId", "1").await.expect("ok");

    // Reopening must find version 1 and the existing store, not recreate them.
    let factory = web_sys::window()
        .expect("window")
        .indexed_db()
        .expect("accessible")
        .expect("present");
    let open = factory.open_with_u32(&name, 1).expect("open");
    let db: web_sys::IdbDatabase = crate::idb::await_request(open.unchecked_into())
        .await
        .expect("opened")
        .dyn_into()
        .expect("database");

    assert_eq!(db.version(), 1.0);
    assert!(db.object_store_names().contains(STORE_NAME));
    db.close();
}

#[wasm_bindgen_test]
async fn the_bucketing_layer_works_over_real_indexeddb() {
    let name = unique_db("bucketed");
    let storage = WeekBucketed::new(IdbStorage::with_database(&name));

    // Seed through the raw layer, then read through the aggregate view.
    let snapshot = snapshot();
    for (key, entry) in snapshot["entries"].as_object().expect("entries") {
        let json = serde_json::to_string(&entry["value"]).expect("serializes");
        storage.inner().set_item(key, &json).await.expect("seeded");
    }

    let aggregate = storage
        .get_item("activities")
        .await
        .expect("ok")
        .expect("aggregate present");
    let activities: Vec<trainer_core::models::Activity> =
        serde_json::from_str(&aggregate).expect("parses");
    assert_eq!(activities.len(), 527);

    let weeks = storage.available_week_keys().await.expect("ok");
    assert_eq!(weeks.len(), 31);
}
