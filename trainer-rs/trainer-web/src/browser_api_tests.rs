//! Browser-tier coverage for the APIs that replaced the JavaScript shims.
//! Task 4.5, plus task 4.3's regression test for issue #85.

use crate::geolocation::{self, GeolocationError};
use crate::notifications;
use crate::scroll::ScrollTrigger;
use js_sys::Array;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::{Element, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

wasm_bindgen_test_configure!(run_in_browser);

async fn sleep_ms(millis: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis);
        }
    });
    let _ = JsFuture::from(promise).await;
}

/// A visible element in the document, which an observer will report as
/// intersecting.
fn visible_element(id: &str) -> Element {
    let document = web_sys::window()
        .expect("window")
        .document()
        .expect("document");
    let element = document.create_element("div").expect("div");
    element.set_id(id);
    element
        .set_attribute("style", "width:50px;height:50px;")
        .expect("style");
    document
        .body()
        .expect("body")
        .append_child(&element)
        .expect("append");
    element
}

fn unique_id(label: &str) -> String {
    format!("trigger-{label}-{}", js_sys::Date::now() as u64)
}

// ── IntersectionObserver ────────────────────────────────────────────────

#[wasm_bindgen_test]
async fn the_trigger_fires_when_the_element_is_visible() {
    let id = unique_id("visible");
    visible_element(&id);

    let count = Rc::new(Cell::new(0u32));
    let counter = count.clone();
    let trigger = ScrollTrigger::new(move || counter.set(counter.get() + 1)).expect("observer");

    trigger.observe(&id);
    sleep_ms(300).await;

    assert!(
        count.get() >= 1,
        "an intersecting element must fire the callback"
    );
}

/// Task 4.3. Re-observing must re-fire, which is what keeps a sparse filtered
/// list loading.
#[wasm_bindgen_test]
async fn re_observing_fires_again() {
    let id = unique_id("rearm");
    visible_element(&id);

    let count = Rc::new(Cell::new(0u32));
    let counter = count.clone();
    let trigger = ScrollTrigger::new(move || counter.set(counter.get() + 1)).expect("observer");

    trigger.observe(&id);
    sleep_ms(300).await;
    let after_first = count.get();

    // The element never left the viewport. Without the unobserve inside
    // observe(), this second call would be a no-op.
    trigger.observe(&id);
    sleep_ms(300).await;

    assert!(
        count.get() > after_first,
        "re-observing must deliver a fresh entry (was {after_first}, now {})",
        count.get()
    );
}

/// The contrast that shows the workaround is load-bearing rather than
/// defensive: a raw observer, observed twice without unobserving, fires once.
#[wasm_bindgen_test]
async fn a_raw_observer_does_not_re_fire_without_the_unobserve() {
    let id = unique_id("raw");
    let element = visible_element(&id);

    let count = Rc::new(Cell::new(0u32));
    let counter = count.clone();
    let callback = Closure::<dyn FnMut(Array)>::new(move |entries: Array| {
        for entry in entries.iter() {
            if let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>()
                && entry.is_intersecting()
            {
                counter.set(counter.get() + 1);
            }
        }
    });

    let options = IntersectionObserverInit::new();
    options.set_root_margin("0px");
    let thresholds = Array::new();
    thresholds.push(&JsValue::from_f64(0.1));
    options.set_threshold(&thresholds);

    let observer =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
            .expect("observer");

    observer.observe(&element);
    sleep_ms(300).await;
    let after_first = count.get();

    // No unobserve: this is the bug issue #85 describes.
    observer.observe(&element);
    sleep_ms(300).await;

    assert_eq!(
        count.get(),
        after_first,
        "observing an already-observed element must be a no-op — if this ever \
         changes, the unobserve in ScrollTrigger::observe is no longer needed"
    );

    observer.disconnect();
    drop(callback);
}

#[wasm_bindgen_test]
async fn observing_a_missing_element_is_ignored() {
    let trigger = ScrollTrigger::new(|| unreachable!("must not fire")).expect("observer");
    trigger.observe("no-such-element");
    sleep_ms(150).await;
}

#[wasm_bindgen_test]
async fn dropping_the_trigger_stops_callbacks() {
    let id = unique_id("drop");
    visible_element(&id);

    let count = Rc::new(Cell::new(0u32));
    let counter = count.clone();
    {
        let trigger = ScrollTrigger::new(move || counter.set(counter.get() + 1)).expect("observer");
        trigger.observe(&id);
        sleep_ms(300).await;
    } // dropped here, which disconnects

    let after_drop = count.get();
    sleep_ms(300).await;
    assert_eq!(
        count.get(),
        after_drop,
        "a dropped trigger must not keep firing"
    );
}

// ── Geolocation ─────────────────────────────────────────────────────────

/// Headless Chrome grants no geolocation permission, so this exercises the
/// failure path: it must resolve to an error rather than hang or panic. The
/// shim never rejected either — it resolved with an error marker.
#[wasm_bindgen_test]
async fn geolocation_resolves_to_an_error_rather_than_hanging() {
    let result = geolocation::current_position().await;
    assert!(
        matches!(
            result,
            Err(GeolocationError::Denied) | Err(GeolocationError::Unavailable)
        ),
        "expected a classified error, got {result:?}"
    );
}

// ── Notifications ───────────────────────────────────────────────────────

/// Without permission every call must return quietly, because the activity
/// timer has to keep working whether or not notifications do.
#[wasm_bindgen_test]
async fn notifications_are_a_silent_no_op_without_permission() {
    notifications::request_permission().await;
    notifications::show(1, "Running", "1:30").await;
    notifications::close(1).await;
}

/// The `data` payload is what makes a notification click navigate.
///
/// The service worker reads `data.activityId`; nothing ever set it, so the
/// handler's navigate branch was unreachable and a click only focused the
/// window. Asserting the shape here is the only way to catch it silently going
/// missing again — a notification that shows correctly gives no sign that its
/// click target is gone.
#[wasm_bindgen_test]
fn the_notification_payload_carries_the_activity_id() {
    let data = notifications::activity_data(42);
    let read = js_sys::Reflect::get(&data, &wasm_bindgen::JsValue::from_str("activityId"))
        .expect("data is an object");
    assert_eq!(read.as_f64(), Some(42.0));
}
