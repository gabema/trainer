//! Ports `Trainer/Pages/ActivityTypeEntry.razor`.
//!
//! As with the other two entry pages, the `EditForm` /
//! `DataAnnotationsValidator` / `ValidationMessage` trio is not ported:
//! `ActivityType` carries no validation attributes, so it validated nothing and
//! `OnValidSubmit` fired unconditionally.

use crate::routes::return_route;
use crate::state::storage;
use crate::views::decimal_input::DecimalAmountInput;
use dioxus::prelude::*;
use trainer_core::helpers::amount::should_warn_about_decimal_places;
use trainer_core::models::{ActivityType, NetBenefit};
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;

const MAX_DECIMAL_PLACES: i32 = 3;

/// The three benefits and the Bootstrap variant each uses when selected and
/// when not.
const BENEFITS: [(NetBenefit, &str, &str, &str); 3] = [
    (
        NetBenefit::Positive,
        "Positive",
        "btn-success",
        "btn-outline-success",
    ),
    (
        NetBenefit::Neutral,
        "Neutral",
        "btn-secondary",
        "btn-outline-secondary",
    ),
    (
        NetBenefit::Negative,
        "Negative",
        "btn-danger",
        "btn-outline-danger",
    ),
];

/// `new ActivityType()` in the C#.
fn blank() -> ActivityType {
    ActivityType {
        id: 0,
        name: String::new(),
        net_benefit: NetBenefit::Neutral,
        daily_amount: None,
        weekly_amount: None,
        unit: None,
        is_private: false,
        decimal_places: 0,
    }
}

#[component]
pub fn ActivityTypeNew(return_to: Option<i32>) -> Element {
    rsx! {
        ActivityTypeForm { id: None, return_to }
    }
}

#[component]
pub fn ActivityTypeEdit(id: i32, return_to: Option<i32>) -> Element {
    rsx! {
        ActivityTypeForm { id: Some(id), return_to }
    }
}

#[component]
fn ActivityTypeForm(id: Option<i32>, return_to: Option<i32>) -> Element {
    let mut activity_type = use_signal(blank);
    // The precision as last saved, and how many activities would be reread if
    // it changed. Both are zero for a new type, which is why the warning cannot
    // fire there.
    let mut saved_decimal_places = use_signal(|| 0);
    let mut activity_count = use_signal(|| 0);
    let navigator = use_navigator();

    use_future(move || async move {
        let Some(id) = id else { return };
        let store = storage();
        if let Ok(Some(existing)) = ActivityTypeService::new(store.inner()).by_id(id).await {
            saved_decimal_places.set(existing.decimal_places);
            activity_type.set(existing);
        }
        if let Ok(activities) = ActivityService::new(&store).by_activity_type_id(id).await {
            activity_count.set(activities.len() as i32);
        }
    });

    let editing = id.is_some();
    let title = if editing {
        "Edit Activity Type"
    } else {
        "Add Activity Type"
    };
    let current = activity_type();
    let show_warning = should_warn_about_decimal_places(
        saved_decimal_places(),
        current.decimal_places,
        activity_count(),
    );

    rsx! {
        div { class: "header-area",
            h1 { class: "mb-0", "{title}" }
        }

        div { class: "container mt-4",
            form {
                onsubmit: move |event| async move {
                    event.prevent_default();
                    let store = storage();
                    let service = ActivityTypeService::new(store.inner());
                    let _ = if editing {
                        service.update(activity_type()).await
                    } else {
                        service.add(activity_type()).await.map(|_| ())
                    };
                    navigator.push(return_route(return_to));
                },

                div { class: "mb-3",
                    label { class: "form-label", "Name" }
                    input {
                        r#type: "text",
                        class: "form-control",
                        value: "{current.name}",
                        oninput: move |event| activity_type.write().name = event.value(),
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Net Benefit" }
                    div { class: "btn-group mt-2", role: "group",
                        for (benefit , label , selected , unselected) in BENEFITS {
                            button {
                                r#type: "button",
                                class: if current.net_benefit == benefit { "btn {selected}" } else { "btn {unselected}" },
                                onclick: move |_| activity_type.write().net_benefit = benefit,
                                "{label}"
                            }
                        }
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Decimal Places" }
                    if show_warning {
                        div { class: "alert alert-warning py-2 small mb-2", role: "alert",
                            "Changing this will reinterpret all {activity_count()} existing "
                            {if activity_count() == 1 { "activity" } else { "activities" }}
                            " of this type (e.g. a stored amount of 125 shows as 125 at 0 places but 1.25 at 2 places)."
                        }
                    }
                    select {
                        class: "form-select",
                        value: "{current.decimal_places}",
                        onchange: move |event| {
                            if let Ok(places) = event.value().parse() {
                                activity_type.write().decimal_places = places;
                            }
                        },
                        for places in 0..=MAX_DECIMAL_PLACES {
                            option { value: "{places}", "{places}" }
                        }
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Daily Amount" }
                    DecimalAmountInput {
                        value: current.daily_amount,
                        decimal_places: current.decimal_places,
                        on_change: move |value| activity_type.write().daily_amount = value,
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Weekly Amount" }
                    DecimalAmountInput {
                        value: current.weekly_amount,
                        decimal_places: current.decimal_places,
                        on_change: move |value| activity_type.write().weekly_amount = value,
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Unit of Measurement" }
                    input {
                        r#type: "text",
                        class: "form-control",
                        placeholder: "e.g., cups, reps, ounces",
                        value: current.unit.clone().unwrap_or_default(),
                        // `InputText` binds a `string?` and writes "" for an
                        // empty field, so an emptied unit is stored as an empty
                        // string rather than null. Kept, because the storage
                        // format distinguishes the two.
                        oninput: move |event| activity_type.write().unit = Some(event.value()),
                    }
                }

                div { class: "mb-3",
                    div { class: "form-check",
                        input {
                            r#type: "checkbox",
                            class: "form-check-input",
                            id: "isPrivate",
                            checked: current.is_private,
                            onchange: move |event| {
                                activity_type.write().is_private = event.checked();
                            },
                        }
                        label { class: "form-check-label", r#for: "isPrivate",
                            "Private (hidden from home, activities, and calendar unless searched by name)"
                        }
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
