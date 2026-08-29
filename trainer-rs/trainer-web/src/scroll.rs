//! Infinite-scroll trigger, replacing `wwwroot/js/infinite-scroll.js`.
//!
//! # The issue #85 workaround
//!
//! `observe()` on an already-observed target is a no-op — the browser will not
//! re-deliver an entry for it. When a filtered list is sparse the trigger
//! element stays inside the viewport after a load, so nothing new intersects
//! and the callback never fires again: loading stalls with the page unscrollable.
//!
//! [`ScrollTrigger::observe`] therefore unobserves first, which forces a fresh
//! intersection evaluation and re-delivers an entry. Losing that line silently
//! breaks "keep loading weeks until results fill the view".

use js_sys::Array;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

/// Matches the shim: viewport root, no margin, fire at 10% visibility.
const THRESHOLD: f64 = 0.1;
const ROOT_MARGIN: &str = "0px";

/// An observer plus the closure it calls.
///
/// The closure must outlive the observer, so it is owned here rather than
/// `forget()`-ten. Dropping this disconnects, which is what the shim's
/// `dispose()` did.
pub struct ScrollTrigger {
    observer: IntersectionObserver,
    _callback: Closure<dyn FnMut(Array)>,
}

impl ScrollTrigger {
    /// Creates an observer that calls `on_visible` whenever the observed
    /// element intersects the viewport.
    pub fn new<F>(mut on_visible: F) -> Option<Self>
    where
        F: FnMut() + 'static,
    {
        let callback = Closure::<dyn FnMut(Array)>::new(move |entries: Array| {
            for entry in entries.iter() {
                if let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>()
                    && entry.is_intersecting()
                {
                    on_visible();
                }
            }
        });

        let options = IntersectionObserverInit::new();
        options.set_root_margin(ROOT_MARGIN);
        let thresholds = Array::new();
        thresholds.push(&JsValue::from_f64(THRESHOLD));
        options.set_threshold(&thresholds);

        let observer =
            IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
                .ok()?;

        Some(Self {
            observer,
            _callback: callback,
        })
    }

    /// Observes the element, re-arming it if it was already observed.
    ///
    /// The unobserve is the issue #85 fix; see the module docs.
    pub fn observe(&self, element_id: &str) {
        let Some(element) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(element_id))
        else {
            return;
        };

        self.observer.unobserve(&element);
        self.observer.observe(&element);
    }

    pub fn disconnect(&self) {
        self.observer.disconnect();
    }
}

impl Drop for ScrollTrigger {
    fn drop(&mut self) {
        self.disconnect();
    }
}
