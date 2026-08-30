//! Ports `Trainer/Pages/KnownLocationEntry.razor`.
//!
//! # Two pieces of C# machinery that are gone
//!
//! * `IsEditMode` re-parsed `NavigationManager.Uri`, split the path, and tried
//!   `int.TryParse` on the last segment — because a location id is a hash and
//!   can be negative, so the usual `Id > 0` test was wrong. Here "new" and
//!   "edit" are separate route variants, so the question does not arise.
//! * `EditForm` with `DataAnnotationsValidator` and a `ValidationMessage` for
//!   `Name`. `KnownLocation` declares no validation attributes, so nothing was
//!   ever validated and `OnValidSubmit` always fired. A plain form submit is
//!   exactly equivalent.

use crate::geolocation::{GeolocationError, current_position};
use crate::routes::return_route;
use crate::state::storage;
use dioxus::prelude::*;
use trainer_core::models::KnownLocation;
use trainer_core::services::known_location::KnownLocationService;

/// The three messages the C# produced, kept verbatim.
const DENIED: &str =
    "Location access was denied. Enable it in browser settings or enter coordinates manually.";
const UNAVAILABLE: &str = "Location is unavailable on this device or browser.";

#[component]
pub fn KnownLocationNew(return_to: Option<i32>) -> Element {
    rsx! {
        KnownLocationForm { id: None, return_to }
    }
}

#[component]
pub fn KnownLocationEdit(id: i32, return_to: Option<i32>) -> Element {
    rsx! {
        KnownLocationForm { id: Some(id), return_to }
    }
}

#[component]
fn KnownLocationForm(id: Option<i32>, return_to: Option<i32>) -> Element {
    // `new KnownLocation()` in the C#: an unsaved location with a zero id and
    // coordinates at the origin, which the form then fills in.
    let mut location = use_signal(|| KnownLocation {
        id: 0,
        name: String::new(),
        latitude: 0.0,
        longitude: 0.0,
    });
    let mut getting_location = use_signal(|| false);
    let mut location_error = use_signal(|| None::<&'static str>);
    let navigator = use_navigator();

    // The C# fetched every location and searched it for the id, because
    // KnownLocationService has no by-id lookup.
    use_future(move || async move {
        let Some(id) = id else { return };
        let store = storage();
        if let Ok(all) = KnownLocationService::new(store.inner()).all().await
            && let Some(existing) = all.into_iter().find(|l| l.id == id)
        {
            location.set(existing);
        }
    });

    let editing = id.is_some();
    let title = if editing {
        "Edit Location"
    } else {
        "Add Location"
    };

    rsx! {
        div { class: "header-area",
            h1 { class: "mb-0", "{title}" }
        }

        div { class: "container mt-4",
            form {
                onsubmit: move |event| async move {
                    event.prevent_default();
                    let store = storage();
                    let _ = KnownLocationService::new(store.inner()).save(location()).await;
                    navigator.push(return_route(return_to));
                },

                div { class: "mb-3",
                    label { class: "form-label", "Name" }
                    input {
                        r#type: "text",
                        class: "form-control",
                        placeholder: "e.g., Home, Gym, Park",
                        value: "{location().name}",
                        oninput: move |event| location.write().name = event.value(),
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Coordinates" }
                    div { class: "row g-2 mb-2",
                        div { class: "col-md-6",
                            label { class: "form-label small text-muted", "Latitude" }
                            input {
                                r#type: "number",
                                class: "form-control",
                                placeholder: "e.g., 37.77493",
                                value: "{location().latitude}",
                                oninput: move |event| {
                                    if let Ok(value) = event.value().parse() {
                                        location.write().latitude = value;
                                    }
                                },
                            }
                        }
                        div { class: "col-md-6",
                            label { class: "form-label small text-muted", "Longitude" }
                            input {
                                r#type: "number",
                                class: "form-control",
                                placeholder: "e.g., -122.41942",
                                value: "{location().longitude}",
                                oninput: move |event| {
                                    if let Ok(value) = event.value().parse() {
                                        location.write().longitude = value;
                                    }
                                },
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-outline-secondary",
                        disabled: getting_location(),
                        onclick: move |_| async move {
                            getting_location.set(true);
                            location_error.set(None);
                            match current_position().await {
                                Ok(coordinates) => {
                                    let mut location = location.write();
                                    location.latitude = coordinates.latitude;
                                    location.longitude = coordinates.longitude;
                                }
                                Err(GeolocationError::Denied) => location_error.set(Some(DENIED)),
                                Err(GeolocationError::Unavailable) => {
                                    location_error.set(Some(UNAVAILABLE))
                                }
                            }
                            getting_location.set(false);
                        },
                        if getting_location() {
                            span {
                                class: "spinner-border spinner-border-sm me-1",
                                role: "status",
                                "aria-hidden": "true",
                            }
                            span { "Getting location..." }
                        } else {
                            span { "Use My Location" }
                        }
                    }
                    if let Some(error) = location_error() {
                        div { class: "text-danger small mt-1", "{error}" }
                    }
                }

                div { class: "mb-3",
                    button { r#type: "submit", class: "btn btn-primary",
                        {if editing { "Update" } else { "Add" }}
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-secondary ms-2",
                        onclick: move |_| {
                            navigator.push(return_route(return_to));
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}
