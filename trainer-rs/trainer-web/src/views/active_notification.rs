//! Headless component bridging active-activity state to browser notifications,
//! ported from `Trainer/Components/ActiveActivityNotification.razor`.
//!
//! Renders nothing. The C# subscribed to `OnChanged` and `OnSlowTick` and
//! unsubscribed in `Dispose`; here the two effects read the signals they depend
//! on and Dioxus re-runs them when those change.

use crate::clock::now_local;
use crate::notifications;
use crate::state::{active_activities, storage};
use dioxus::prelude::*;
use std::collections::BTreeMap;
use trainer_core::helpers::when::format_elapsed;
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;

/// Falls back to "Activity" where the C# did, so a notification still appears
/// when the type cannot be resolved.
async fn resolve_names(ids: Vec<i32>) -> BTreeMap<i32, String> {
    let mut names = BTreeMap::new();
    if ids.is_empty() {
        return names;
    }

    let store = storage();
    let types = ActivityTypeService::new(store.inner())
        .all()
        .await
        .unwrap_or_default();
    let activities = ActivityService::new(&store);

    for id in ids {
        let name = match activities.by_id(id).await {
            Ok(Some(activity)) => types
                .iter()
                .find(|t| t.id == activity.activity_type_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Activity".to_owned()),
            _ => "Activity".to_owned(),
        };
        names.insert(id, name);
    }
    names
}

#[component]
pub fn ActiveActivityNotification() -> Element {
    let state = active_activities();

    // Ask once on mount. Already-granted and already-denied both short-circuit.
    use_future(|| async move {
        notifications::request_permission().await;
    });

    // Replaces OnChanged: show notifications for newly active activities and
    // close them for ones that ended. Tracking the previous set here is what
    // the C# used its _typeNames dictionary for.
    let mut shown = use_signal(BTreeMap::<i32, String>::new);
    let entries = state.entries();
    let ids: Vec<i32> = entries.keys().copied().collect();

    use_effect(use_reactive!(|ids| {
        spawn(async move {
            let current: Vec<i32> = ids.clone();
            let previous = shown.peek().clone();

            for stale in previous.keys().filter(|id| !current.contains(id)) {
                notifications::close(*stale).await;
            }

            let names = resolve_names(current.clone()).await;
            let now = now_local();
            let active = active_activities().entries();

            for id in &current {
                if previous.contains_key(id) {
                    continue;
                }
                let name = names
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| "Activity".to_owned());
                if let Some(start) = active.get(id) {
                    let elapsed = format_elapsed(now - start.naive());
                    notifications::show(*id, &name, &elapsed).await;
                }
            }

            shown.set(names);
        });
    }));

    // Replaces OnSlowTick: refresh every notification's elapsed time. Reading
    // slow_tick is what subscribes this effect to the thirty-second clock.
    let slow = state.slow_tick();
    use_effect(use_reactive!(|slow| {
        // Skip the initial run; the effect above has just shown them.
        if slow == 0 {
            return;
        }
        spawn(async move {
            let names = shown.peek().clone();
            let now = now_local();
            for (id, start) in active_activities().entries() {
                let name = names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| "Activity".to_owned());
                notifications::show(id, &name, &format_elapsed(now - start.naive())).await;
            }
        });
    }));

    rsx! {}
}
