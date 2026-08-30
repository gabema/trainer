//! Shared reactive state, replacing the C# event plumbing.
//!
//! `ActiveActivityService` exposed `OnChanged`, `OnTick` and `OnSlowTick`, and
//! each of the three consuming components subscribed in `OnInitialized`,
//! unsubscribed in `Dispose`, and wrapped every callback in
//! `InvokeAsync(StateHasChanged)`.
//!
//! Here the active set is a [`Signal`]. Reading it in a component subscribes
//! that component automatically, and writing it re-renders every reader — so
//! there is no subscription, no unsubscription, and no manual redraw anywhere.
//!
//! The timers survive, because elapsed time genuinely has to re-render on a
//! clock rather than on a state change. They run only while something is
//! active, matching `EnsureTimersRunning` / `StopTimers`.

use crate::local::LocalStorage;
use dioxus::prelude::*;
use std::collections::BTreeMap;
use trainer_core::services::active_activity::ActiveActivityService;
use trainer_core::services::active_time::ActiveTime;

/// `LocalStorage` is a zero-sized handle, so one static instance can back the
/// service for the life of the app.
static LOCAL_STORAGE: LocalStorage = LocalStorage;

/// Builds a service over the current stored state.
///
/// Each operation is a fresh read-modify-write rather than a long-lived
/// instance. The service keeps its own set in a `RefCell`, which would be a
/// second source of truth alongside the signal; loading it per call keeps
/// localStorage authoritative for persistence and the signal authoritative for
/// rendering, with nothing to drift between them.
async fn load_service() -> ActiveActivityService<'static, LocalStorage> {
    let service = ActiveActivityService::new(&LOCAL_STORAGE);
    // Corrupt state is cleared by the service rather than surfaced.
    let _ = service.initialize().await;
    service
}

/// Handle to the active-activity state. `Copy`, so components take it from
/// context and use it without cloning ceremony.
#[derive(Clone, Copy)]
pub struct ActiveActivities {
    /// The active set. Any component reading this re-renders when it changes.
    entries: Signal<BTreeMap<i32, ActiveTime>>,
    /// Advances once a second, for elapsed-time displays. Replaces `OnTick`.
    tick: Signal<u64>,
    /// Advances every thirty seconds, for notification refresh. Replaces
    /// `OnSlowTick`.
    slow_tick: Signal<u64>,
}

impl ActiveActivities {
    /// The current active set. Reading subscribes the calling component.
    pub fn entries(&self) -> BTreeMap<i32, ActiveTime> {
        self.entries.read().clone()
    }

    pub fn is_active(&self, activity_id: i32) -> bool {
        self.entries.read().contains_key(&activity_id)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Reading this subscribes to the one-second clock without depending on the
    /// active set itself.
    pub fn tick(&self) -> u64 {
        *self.tick.read()
    }

    pub fn slow_tick(&self) -> u64 {
        *self.slow_tick.read()
    }

    /// Loads persisted state. Corrupt state is discarded silently by the
    /// service, so this cannot fail the app's startup.
    pub async fn initialize(&mut self) {
        let service = load_service().await;
        self.entries.set(service.all());
    }

    pub async fn start(&mut self, activity_id: i32, start_time: ActiveTime) {
        let service = load_service().await;
        let _ = service.start(activity_id, start_time).await;
        self.entries.set(service.all());
    }

    pub async fn finish(&mut self, activity_id: i32) {
        let service = load_service().await;
        let _ = service.finish(activity_id).await;
        self.entries.set(service.all());
    }
}

/// Installs the state into context and starts the clocks. Called once, at the
/// app root.
pub fn use_active_activities() -> ActiveActivities {
    let state = use_context_provider(|| ActiveActivities {
        entries: Signal::new(BTreeMap::new()),
        tick: Signal::new(0),
        slow_tick: Signal::new(0),
    });

    // Load persisted state once.
    use_future(move || async move {
        let mut state = state;
        state.initialize().await;
    });

    // The two clocks. Each advances only while something is active, so an idle
    // app does no work — the behaviour `EnsureTimersRunning` / `StopTimers` had.
    use_future(move || async move {
        let mut state = state;
        loop {
            sleep_ms(1_000).await;
            if !state.entries.peek().is_empty() {
                let next = *state.tick.peek() + 1;
                state.tick.set(next);
            }
        }
    });

    use_future(move || async move {
        let mut state = state;
        loop {
            sleep_ms(30_000).await;
            if !state.entries.peek().is_empty() {
                let next = *state.slow_tick.peek() + 1;
                state.slow_tick.set(next);
            }
        }
    });

    state
}

/// Reads the state a parent installed.
pub fn active_activities() -> ActiveActivities {
    use_context::<ActiveActivities>()
}

/// Awaits a browser timeout.
///
/// Uses `setTimeout` through a `Promise`, so the resolve callback is already a
/// JS function and no `Closure` lifetime has to be managed — unlike the
/// `IdbRequest` adapter, which had no promise to wrap.
async fn sleep_ms(millis: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// The storage stack the services run over.
///
/// Both layers are cheap handles rather than open connections, so building one
/// per operation is fine and avoids sharing mutable state across components.
pub fn storage() -> trainer_core::storage::WeekBucketed<crate::idb::IdbStorage> {
    trainer_core::storage::WeekBucketed::new(crate::idb::IdbStorage::new())
}
