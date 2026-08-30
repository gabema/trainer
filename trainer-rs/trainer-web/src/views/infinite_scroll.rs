//! Wires the section-4 [`ScrollTrigger`] into a component.
//!
//! # Why a channel sits in the middle
//!
//! The observer callback is invoked by the browser, not by Dioxus, so it runs
//! with no reactive runtime in scope and must not touch signals. It sends on a
//! plain channel instead; the component drains that channel from inside a
//! `use_future`, where writing signals and spawning work is ordinary.
//!
//! The C# had the same boundary and crossed it with a `DotNetObjectReference`
//! plus a `[JSInvokable]` method — a round trip through the JS interop layer for
//! what is here a `send` on an unbounded queue.
//!
//! # Re-arming
//!
//! `observe` must be called again after each render that keeps the trigger in
//! the tree: a trigger that never leaves the viewport (sparse search results)
//! would otherwise fire once and stop, which is issue #85. `ScrollTrigger`
//! unobserves before observing for exactly that reason. The C# also slept 100ms
//! first, to let Blazor commit the DOM; Dioxus effects already run after commit,
//! so the sleep has nothing left to wait for.

use crate::scroll::ScrollTrigger;
use dioxus::prelude::*;
use futures_channel::mpsc::{UnboundedReceiver, unbounded};

/// Handle to a component's scroll trigger. `Copy`, so it can move into futures
/// and effects without cloning ceremony.
#[derive(Clone, Copy)]
pub struct ScrollLoader {
    /// Dropped with the owning scope, which disconnects the observer.
    trigger: CopyValue<Option<ScrollTrigger>>,
    /// Taken once, by the task that drains it.
    requests: CopyValue<Option<UnboundedReceiver<()>>>,
}

impl ScrollLoader {
    /// Points the observer at the trigger element, re-arming it if it was
    /// already watching that element.
    pub fn observe(&self, element_id: &str) {
        if let Some(trigger) = self.trigger.read().as_ref() {
            trigger.observe(element_id);
        }
    }

    /// The stream of "trigger came into view" notifications. Yields `Some` to
    /// the first caller and `None` after that, so the draining task cannot be
    /// started twice.
    pub fn requests(&mut self) -> Option<UnboundedReceiver<()>> {
        self.requests.write().take()
    }
}

pub fn use_scroll_loader() -> ScrollLoader {
    use_hook(|| {
        let (sender, receiver) = unbounded();
        let trigger = ScrollTrigger::new(move || {
            // The only thing the browser-side callback does. A send on a
            // disconnected channel is not an error worth reporting: it means
            // the page is being torn down.
            let _ = sender.unbounded_send(());
        });
        ScrollLoader {
            trigger: CopyValue::new(trigger),
            requests: CopyValue::new(Some(receiver)),
        }
    })
}
