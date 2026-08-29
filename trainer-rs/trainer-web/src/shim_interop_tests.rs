//! Task 9.3 — data written by Rust must be readable by the JavaScript shim that
//! ships today, and vice versa.
//!
//! Rather than assert about the shim's behavior, this loads
//! `Trainer/wwwroot/js/indexeddb-storage.js` verbatim into the test page and
//! drives it. Its `DB_NAME` constant is rewritten first so a test run cannot
//! touch a real `Trainer` database.

use crate::idb::IdbStorage;
use js_sys::{Function, Object, Promise, Reflect};
use trainer_core::storage::Storage;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

/// The shipping shim, embedded at compile time so the test cannot drift from it.
const SHIM_SOURCE: &str = include_str!("../../../Trainer/wwwroot/js/indexeddb-storage.js");

/// Loads the shim against an isolated database and returns its object.
fn load_shim(db_name: &str) -> Object {
    let patched = SHIM_SOURCE.replace(
        "const DB_NAME = 'Trainer';",
        &format!("const DB_NAME = '{db_name}';"),
    );
    assert!(
        patched != SHIM_SOURCE,
        "the shim's DB_NAME declaration changed; update this test before it \
         starts writing to the real Trainer database"
    );

    js_sys::eval(&patched).expect("the shim evaluates");

    let window = web_sys::window().expect("window");
    Reflect::get(&window, &JsValue::from_str("indexedDbStorage"))
        .expect("the shim exposes indexedDbStorage")
        .dyn_into::<Object>()
        .expect("indexedDbStorage is an object")
}

async fn shim_call(shim: &Object, method: &str, arg: &JsValue) -> JsValue {
    let function: Function = Reflect::get(shim, &JsValue::from_str(method))
        .unwrap_or_else(|_| panic!("the shim exposes {method}"))
        .dyn_into()
        .expect("a function");
    let promise: Promise = function
        .call1(shim, arg)
        .expect("the call succeeds")
        .dyn_into()
        .expect("a promise");
    JsFuture::from(promise).await.expect("the promise resolves")
}

fn unique_db(label: &str) -> String {
    format!("TrainerShim-{label}-{}", js_sys::Date::now() as u64)
}

#[wasm_bindgen_test]
async fn the_shim_reads_what_rust_wrote() {
    let name = unique_db("rust-to-js");
    let storage = IdbStorage::with_database(&name);

    // Written through the Rust storage layer, in the storage format.
    let bucket = r#"[{"id":1,"activityTypeId":2,"when":"2026-01-01T08:56:44-08","amount":15,"notes":"hello","durationSeconds":null,"knownLocationId":null}]"#;
    storage
        .set_item("activities-2026.01", bucket)
        .await
        .expect("written");
    storage
        .set_item("activityNextId", "536")
        .await
        .expect("written");

    let shim = load_shim(&name);

    // The shim JSON.stringifies whatever it finds, so a match proves the stored
    // representation is the one it expects.
    let read = shim_call(&shim, "getItem", &JsValue::from_str("activities-2026.01")).await;
    let text = read.as_string().expect("the shim returns a string");
    let from_shim: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    let expected: serde_json::Value = serde_json::from_str(bucket).expect("valid JSON");
    assert_eq!(from_shim, expected, "the shim must see what Rust wrote");

    // The scalar key too, which is where the object-vs-string question bites.
    let scalar = shim_call(&shim, "getItem", &JsValue::from_str("activityNextId")).await;
    assert_eq!(scalar.as_string().as_deref(), Some("536"));
}

#[wasm_bindgen_test]
async fn rust_reads_what_the_shim_wrote() {
    let name = unique_db("js-to-rust");
    let shim = load_shim(&name);

    // Write through the shim's own setItem, which JSON.parses before storing.
    let payload = r#"[{"id":7,"activityTypeId":1,"when":"2026-06-15T10:00:00Z","amount":3,"notes":null,"durationSeconds":90,"knownLocationId":null}]"#;
    let set: Function = Reflect::get(&shim, &JsValue::from_str("setItem"))
        .expect("setItem")
        .dyn_into()
        .expect("a function");
    let promise: Promise = set
        .call2(
            &shim,
            &JsValue::from_str("activities-2026.25"),
            &JsValue::from_str(payload),
        )
        .expect("the call succeeds")
        .dyn_into()
        .expect("a promise");
    JsFuture::from(promise).await.expect("resolves");

    // Read it back through Rust and parse into the real model.
    let storage = IdbStorage::with_database(&name);
    let read = storage
        .get_item("activities-2026.25")
        .await
        .expect("read succeeds")
        .expect("present");
    let activities: Vec<trainer_core::models::Activity> =
        serde_json::from_str(&read).expect("parses into models");

    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id, 7);
    assert_eq!(activities[0].duration_seconds, Some(90));
    assert!(activities[0].notes.is_none());
    assert_eq!(activities[0].when.to_wire(), "2026-06-15T10:00:00Z");
}

#[wasm_bindgen_test]
async fn the_shim_and_rust_agree_on_prefix_search() {
    let name = unique_db("prefix");
    let storage = IdbStorage::with_database(&name);

    for key in ["activities-2026.01", "activities-2026.02", "activityTypes"] {
        storage.set_item(key, "[]").await.expect("written");
    }

    let shim = load_shim(&name);
    let keys = shim_call(
        &shim,
        "getAllKeysWithPrefix",
        &JsValue::from_str("activities-"),
    )
    .await;
    let shim_keys: Vec<String> = js_sys::Array::from(&keys)
        .iter()
        .filter_map(|v| v.as_string())
        .collect();

    let mut rust_keys = storage.keys_with_prefix("activities-").await.expect("ok");
    rust_keys.sort();
    let mut shim_keys = shim_keys;
    shim_keys.sort();

    assert_eq!(rust_keys, shim_keys);
    assert_eq!(rust_keys.len(), 2, "activityTypes must be excluded by both");
}
