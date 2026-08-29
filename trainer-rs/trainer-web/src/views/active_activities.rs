//! The Active Activities section, ported from
//! `Trainer/Components/ActiveActivities.razor`.
//!
//! The C# subscribed to `OnChanged` and `OnTick` in `OnInitialized`,
//! unsubscribed in `Dispose`, and wrapped both callbacks in
//! `InvokeAsync(StateHasChanged)`. None of that exists here: reading the signal
//! subscribes the component, and writing it re-renders.

use crate::clock::now_local;
use crate::state::{ActiveActivities as State, active_activities, storage};
use dioxus::prelude::*;
use std::collections::BTreeMap;
use trainer_core::helpers::display::format_duration;
use trainer_core::helpers::when::format_elapsed;
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;

/// Resolves the activity-type name for each active activity.
///
/// The C# cached names and only re-resolved missing ones, deliberately not
/// caching an empty name so a later attempt could retry once types had loaded.
/// A resource keyed on the active set does the same thing without the cache:
/// it re-runs when the set changes and not otherwise.
async fn resolve_type_names(ids: Vec<i32>) -> BTreeMap<i32, String> {
    let mut names = BTreeMap::new();
    if ids.is_empty() {
        return names;
    }

    let store = storage();
    let Ok(types) = ActivityTypeService::new(store.inner()).all().await else {
        return names;
    };
    let activities = ActivityService::new(&store);

    for id in ids {
        if let Ok(Some(activity)) = activities.by_id(id).await
            && let Some(activity_type) = types.iter().find(|t| t.id == activity.activity_type_id)
        {
            names.insert(id, activity_type.name.clone());
        }
    }
    names
}

/// Writes the elapsed duration onto the activity, saves it, and clears the
/// timer — the whole of `FinishActivityAsync`.
async fn finish_activity(mut state: State, activity_id: i32) {
    let Some(start) = state.entries().get(&activity_id).copied() else {
        return;
    };

    let elapsed = now_local() - start.naive();
    let store = storage();
    let activities = ActivityService::new(&store);

    if let Ok(Some(mut activity)) = activities.by_id(activity_id).await {
        activity.duration_seconds = Some(elapsed.num_seconds().max(0) as i32);
        let _ = activities.update(activity).await;
    }

    state.finish(activity_id).await;
}

#[component]
pub fn ActiveActivities() -> Element {
    let state = active_activities();
    let entries = state.entries();

    let ids: Vec<i32> = entries.keys().copied().collect();
    let names = use_resource(use_reactive!(|ids| resolve_type_names(ids)));

    // Reading the clock subscribes this component to the one-second tick, so
    // the elapsed times below re-render even though the active set has not
    // changed. This is what OnTick did.
    let _tick = state.tick();

    // "The section SHALL only be visible when at least one activity is
    // currently active."
    if entries.is_empty() {
        return rsx! {};
    }

    let resolved = names.read().clone().unwrap_or_default();
    let now = now_local();

    rsx! {
        div { class: "row mb-4",
            div { class: "col-md-12",
                div { class: "card border-warning",
                    div { class: "card-header active-activities-header",
                        h5 { class: "mb-0", "Active Activities" }
                    }
                    div { class: "card-body p-0",
                        ul { class: "list-group list-group-flush",
                            for (activity_id , start_time) in entries {
                                {
                                    // The C# skipped entries whose type name had
                                    // not resolved yet rather than showing a blank row.
                                    let Some(name) = resolved.get(&activity_id).cloned() else {
                                        return rsx! {};
                                    };
                                    let elapsed = format_elapsed(now - start_time.naive());
                                    rsx! {
                                        li {
                                            key: "{activity_id}",
                                            class: "list-group-item d-flex justify-content-between align-items-center",
                                            span {
                                                strong { "{name}" }
                                                span { class: "ms-2 text-muted font-monospace", "{elapsed}" }
                                            }
                                            button {
                                                class: "btn btn-sm btn-success",
                                                onclick: move |_| async move {
                                                    finish_activity(state, activity_id).await;
                                                },
                                                "Finish"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Kept for the activity card in section 5, which shows the same compact
/// duration for a finished activity.
pub fn compact_duration(seconds: Option<i32>) -> Option<String> {
    format_duration(seconds)
}
