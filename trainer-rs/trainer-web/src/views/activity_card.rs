//! Ports `Trainer/Components/ActivityCard.razor`.
//!
//! Tapping the card reveals an overlay of actions; Delete swaps that overlay
//! for a confirmation. Both are local booleans, so both are plain signals — and
//! the `StateHasChanged()` call that followed every one of the C#'s five
//! toggles is gone, along with the `OnChanged` subscribe/unsubscribe pair that
//! existed only to re-render the Finish button.

use crate::routes::Route;
use crate::state::{active_activities, storage};
use crate::views::active_activities::finish_activity;
use dioxus::prelude::*;
use trainer_core::helpers::display::format_activity;
use trainer_core::helpers::when::format_when;
use trainer_core::models::{Activity, ActivityType, KnownLocation};
use trainer_core::services::activity::ActivityService;

#[component]
pub fn ActivityCard(
    activity: Activity,
    activity_types: Vec<ActivityType>,
    known_locations: Vec<KnownLocation>,
    /// Fires after a delete or a finish, so the parent can reload. The C# named
    /// this `OnActivityDeleted` and then also raised it on finish.
    on_changed: EventHandler<()>,
) -> Element {
    let mut show_overlay = use_signal(|| false);
    let mut show_delete_confirmation = use_signal(|| false);
    let state = active_activities();
    let navigator = use_navigator();

    let activity_id = activity.id;
    let type_name = activity_types
        .iter()
        .find(|t| t.id == activity.activity_type_id)
        .map_or("Unknown", |t| t.name.as_str())
        .to_owned();
    let amount = format_activity(&activity, &activity_types, &known_locations);
    let when = format_when(activity.when.naive(), crate::clock::now_local());
    let notes = activity
        .notes
        .as_deref()
        .filter(|n| !n.trim().is_empty())
        .map(str::to_owned);

    rsx! {
        div {
            class: "card activity-card mb-3 position-relative",
            style: "cursor: pointer;",
            // Tapping the card toggles the overlay, but not while the delete
            // confirmation is up.
            onclick: move |_| {
                if !show_delete_confirmation() {
                    show_overlay.toggle();
                }
            },

            div { class: "card-header d-flex justify-content-between align-items-center",
                span { class: "fw-semibold", "{type_name}" }
                span { class: "text-muted small", "{when}" }
            }
            div { class: "card-body",
                div { class: "mb-2",
                    strong { "{amount}" }
                }
                if let Some(notes) = notes {
                    div { class: "activity-notes", "{notes}" }
                }
            }

            if show_overlay() {
                div {
                    class: "activity-card-overlay-backdrop",
                    onclick: move |event| {
                        event.stop_propagation();
                        show_overlay.set(false);
                    },
                }
                div {
                    class: "activity-card-overlay",
                    onclick: move |event| event.stop_propagation(),
                    div { class: "activity-card-overlay-actions",
                        button {
                            class: "btn btn-primary activity-card-overlay-button",
                            title: "Edit",
                            onclick: move |_| {
                                navigator.push(Route::ActivityEdit { id: activity_id });
                            },
                            EditIcon {}
                            span { "Edit" }
                        }
                        if state.is_active(activity_id) {
                            button {
                                class: "btn btn-success activity-card-overlay-button",
                                title: "Finish",
                                onclick: move |_| async move {
                                    finish_activity(state, activity_id).await;
                                    show_overlay.set(false);
                                    on_changed.call(());
                                },
                                ClockIcon {}
                                span { "Finish" }
                            }
                        }
                        button {
                            class: "btn btn-secondary activity-card-overlay-button",
                            title: "Duplicate",
                            onclick: move |_| {
                                navigator
                                    .push(Route::ActivityNew {
                                        duplicate_from: Some(activity_id),
                                    });
                            },
                            DuplicateIcon {}
                            span { "Duplicate" }
                        }
                        button {
                            class: "btn btn-danger activity-card-overlay-button",
                            title: "Delete",
                            onclick: move |_| {
                                show_overlay.set(false);
                                show_delete_confirmation.set(true);
                            },
                            DeleteIcon {}
                            span { "Delete" }
                        }
                    }
                }
            }

            if show_delete_confirmation() {
                div {
                    class: "activity-card-overlay-backdrop",
                    onclick: move |event| {
                        event.stop_propagation();
                        show_delete_confirmation.set(false);
                    },
                }
                div {
                    class: "activity-card-delete-confirmation",
                    onclick: move |event| event.stop_propagation(),
                    div { class: "card",
                        div { class: "card-header",
                            h5 { class: "mb-0", "Confirm Delete" }
                        }
                        div { class: "card-body",
                            p { "Are you sure you want to delete this activity?" }
                        }
                        div { class: "card-footer d-flex justify-content-end gap-2",
                            button {
                                class: "btn btn-secondary",
                                onclick: move |_| show_delete_confirmation.set(false),
                                "Cancel"
                            }
                            button {
                                class: "btn btn-danger",
                                onclick: move |_| async move {
                                    show_delete_confirmation.set(false);
                                    let store = storage();
                                    let _ = ActivityService::new(&store).delete(activity_id).await;
                                    on_changed.call(());
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M12.854.146a.5.5 0 0 0-.707 0L10.5 1.793 14.207 5.5l1.647-1.646a.5.5 0 0 0 0-.708l-3-3zm.646 6.061L9.793 2.5 3.293 9H3.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.207l6.5-6.5zm-7.468 7.468A.5.5 0 0 1 6 13.5V13h-.5a.5.5 0 0 1-.5-.5V12h-.5a.5.5 0 0 1-.5-.5V11h-.5a.5.5 0 0 1-.5-.5V10h-.5a.499.499 0 0 1-.175-.032l-.179.178a.5.5 0 0 0-.11.168l-2 5a.5.5 0 0 0 .65.65l5-2a.5.5 0 0 0 .168-.11l.178-.178z" }
        }
    }
}

#[component]
fn ClockIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z" }
            path { d: "M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z" }
        }
    }
}

#[component]
fn DuplicateIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1z" }
            path { d: "M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0z" }
        }
    }
}

#[component]
fn DeleteIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5m2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5m3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0z" }
            path { d: "M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1zM4.118 4 4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4zM2.5 3h11V2h-11z" }
        }
    }
}
