//! Ports `Trainer/Pages/ActivityEntry.razor`.
//!
//! The `OnChanged` subscription this page kept purely to flip the Start/Stop
//! button is gone: reading the shared active-activity signal subscribes the
//! component, so `_isActive` is derived rather than cached, and `Dispose` has
//! nothing left to unhook.
//!
//! As with the other entry pages, `EditForm` + `DataAnnotationsValidator` +
//! three `ValidationMessage`s are not ported. `Activity` declares no validation
//! attributes, so none of them could ever render.

use crate::clock::{local_when, now_local, now_when};
use crate::geolocation::{GeolocationError, current_position};
use crate::routes::Route;
use crate::state::{active_activities, storage};
use crate::views::decimal_input::DecimalAmountInput;
use chrono::{NaiveDate, NaiveTime};
use dioxus::prelude::*;
use trainer_core::datetime::TrainerTime;
use trainer_core::helpers::duration;
use trainer_core::helpers::strings::null_if_empty_or_whitespace;
use trainer_core::helpers::when::format_elapsed;
use trainer_core::models::{Activity, ActivityType, KnownLocation, duplicate_of};
use trainer_core::services::active_time::ActiveTime;
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;
use trainer_core::services::known_location::KnownLocationService;

const DENIED: &str =
    "Location access was denied. Enable it in browser settings or enter coordinates manually.";
const UNAVAILABLE: &str = "Location is unavailable on this device or browser.";
const UNKNOWN_FAILURE: &str = "Unable to get location. Enter coordinates manually.";

/// The Duration field's text for a stored duration.
///
/// Deliberately not `helpers::display::format_duration`, which renders the
/// card's `5m 30s`. This is the field's own round trip: whatever it produces
/// must parse back through `duration::try_parse` to the same number of seconds.
///
/// ```text
/// None or <= 0 -> ""      an unset duration leaves the field blank
/// 20           -> "0:20"  seconds only, so it cannot read as 20 minutes
/// 1200         -> "20"    whole minutes
/// 330          -> "5:30"
/// ```
fn duration_field_text(duration_seconds: Option<i32>) -> String {
    let Some(total) = duration_seconds.filter(|d| *d > 0) else {
        return String::new();
    };
    let (minutes, seconds) = (total / 60, total % 60);
    match (minutes, seconds) {
        (0, s) => format!("0:{s:02}"),
        (m, 0) => format!("{m}"),
        (m, s) => format!("{m}:{s:02}"),
    }
}

/// Orders locations as `StringComparer.OrdinalIgnoreCase` did. .NET's ordinal
/// ignore-case comparison folds by uppercasing, so this does too.
fn sort_by_name(locations: &mut [KnownLocation]) {
    locations.sort_by_key(|l| l.name.to_uppercase());
}

/// Maps the activity's `When` onto the timer's own wire format.
///
/// .NET distinguishes three kinds and `ActiveActivityService` writes all three
/// differently, but the storage converter behind `TrainerTime` collapses
/// `Local` and `Unspecified` into one, so the bare no-suffix form cannot be
/// reconstructed here. The wall clock is identical either way, and elapsed time
/// reads only the wall clock.
fn timer_start(when: TrainerTime) -> ActiveTime {
    match when {
        TrainerTime::Utc(naive) => ActiveTime::Utc(naive),
        TrainerTime::Offset { naive, offset } => ActiveTime::Offset { naive, offset },
    }
}

/// `new Activity { When = DateTime.Now }`.
fn blank() -> Activity {
    Activity {
        id: 0,
        activity_type_id: 0,
        when: now_when(),
        amount: 0,
        notes: None,
        duration_seconds: None,
        known_location_id: None,
    }
}

#[component]
pub fn ActivityNew(duplicate_from: Option<i32>) -> Element {
    rsx! {
        ActivityForm { id: None, duplicate_from }
    }
}

#[component]
pub fn ActivityEdit(id: i32) -> Element {
    rsx! {
        ActivityForm { id: Some(id), duplicate_from: None }
    }
}

#[component]
fn ActivityForm(id: Option<i32>, duplicate_from: Option<i32>) -> Element {
    let mut activity = use_signal(blank);
    let mut activity_types = use_signal(Vec::<ActivityType>::new);
    let mut known_locations = use_signal(Vec::<KnownLocation>::new);
    let mut selected_location_id = use_signal(|| None::<i32>);
    let mut duration_input = use_signal(String::new);
    let duration_error = use_signal(|| None::<&'static str>);
    let mut getting_location = use_signal(|| false);
    let mut location_error = use_signal(|| None::<&'static str>);
    let mut state = active_activities();
    let navigator = use_navigator();

    use_future(move || async move {
        let store = storage();
        activity_types.set(
            ActivityTypeService::new(store.inner())
                .all()
                .await
                .unwrap_or_default(),
        );
        let mut locations = KnownLocationService::new(store.inner())
            .all()
            .await
            .unwrap_or_default();
        sort_by_name(&mut locations);
        known_locations.set(locations);

        let activities = ActivityService::new(&store);
        if let Some(id) = id {
            if let Ok(Some(existing)) = activities.by_id(id).await {
                activity.set(existing);
            }
        } else if let Some(source_id) = duplicate_from
            && let Ok(Some(source)) = activities.by_id(source_id).await
        {
            activity.set(duplicate_of(&source, now_when()));
        }

        selected_location_id.set(activity().known_location_id);
        duration_input.set(duration_field_text(activity().duration_seconds));
    });

    let editing = id.is_some();
    let current = activity();
    let types = activity_types();
    let locations = known_locations();
    let selected_type = types.iter().find(|t| t.id == current.activity_type_id);
    let decimal_places = selected_type.map_or(0, |t| t.decimal_places);
    let unit_suffix = selected_type
        .and_then(|t| t.unit.as_deref())
        .map(|unit| format!(" ({unit})"))
        .unwrap_or_default();
    let is_active = state.is_active(current.id);
    let when = current.when.naive();
    // Pre-rendered: rsx cannot format-call inside an interpolated attribute.
    let when_date = when.format("%Y-%m-%d").to_string();
    let when_time = when.format("%H:%M").to_string();

    // Where the type and location entry pages come back to. The C# built this
    // as a percent-encoded `returnUrl` query string that only ever held
    // `/activity` or `/activity/{id}`, so the id alone carries it.
    let return_to = id;

    /// Writes the form's fields onto the activity and saves it.
    ///
    /// Returns false when the Duration field does not parse, which is the one
    /// thing that blocks a save.
    async fn save(
        mut activity: Signal<Activity>,
        mut duration_error: Signal<Option<&'static str>>,
        duration_input: Signal<String>,
        selected_location_id: Signal<Option<i32>>,
    ) -> bool {
        let seconds = match duration::try_parse(Some(&duration_input())) {
            Ok(seconds) => seconds,
            Err(error) => {
                duration_error.set(Some(error));
                return false;
            }
        };
        duration_error.set(None);

        {
            let mut activity = activity.write();
            activity.duration_seconds = seconds;
            activity.notes =
                null_if_empty_or_whitespace(activity.notes.as_deref()).map(str::to_owned);
            activity.known_location_id = selected_location_id();
        }

        let store = storage();
        let activities = ActivityService::new(&store);
        let current = activity();
        if current.id > 0 {
            let _ = activities.update(current).await;
        } else if let Ok(saved) = activities.add(current).await {
            // The assigned id matters: starting the timer keys on it.
            activity.set(saved);
        }
        true
    }

    rsx! {
        div { class: "header-area",
            h1 { class: "mb-0", {if editing { "Edit Activity" } else { "Add Activity" }} }
        }

        div { class: "container mt-4",
            form {
                onsubmit: move |event| async move {
                    event.prevent_default();
                    if save(activity, duration_error, duration_input, selected_location_id).await {
                        navigator.push(Route::Home {});
                    }
                },

                div { class: "mb-3",
                    label { class: "form-label", "Type" }
                    div { class: "input-group",
                        select {
                            class: "form-select",
                            value: "{current.activity_type_id}",
                            onchange: move |event| {
                                if let Ok(type_id) = event.value().parse() {
                                    activity.write().activity_type_id = type_id;
                                }
                            },
                            option { value: "0", "Select an activity type..." }
                            for activity_type in types.iter() {
                                option { key: "{activity_type.id}", value: "{activity_type.id}", "{activity_type.name}" }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-outline-secondary",
                            onclick: move |_| {
                                let type_id = activity().activity_type_id;
                                navigator
                                    .push(
                                        if type_id == 0 {
                                            Route::ActivityTypeNew { return_to }
                                        } else {
                                            Route::ActivityTypeEdit { id: type_id, return_to }
                                        },
                                    );
                            },
                            if current.activity_type_id == 0 {
                                span { "+" }
                            } else {
                                PencilIcon {}
                            }
                        }
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "When" }
                    div { class: "row g-2",
                        div { class: "col-md-6",
                            input {
                                r#type: "date",
                                class: "form-control",
                                value: "{when_date}",
                                // Blazor's InputDate<DateTime> parsed the field
                                // with "yyyy-MM-dd" and assigned the result
                                // whole, so picking a date reset the time to
                                // midnight even though a separate time field
                                // sits beside it. Reproduced rather than
                                // quietly corrected.
                                onchange: move |event| {
                                    if let Ok(date) = NaiveDate::parse_from_str(&event.value(), "%Y-%m-%d") {
                                        // `InputDate` parsed into a kind of
                                        // `Unspecified`, which writes with the
                                        // local offset.
                                        activity.write().when = local_when(
                                            date.and_time(NaiveTime::MIN),
                                        );
                                    }
                                },
                            }
                        }
                        div { class: "col-md-6",
                            input {
                                r#type: "time",
                                class: "form-control",
                                value: "{when_time}",
                                onchange: move |event| {
                                    // `TimeSpan.TryParse` accepted "HH:mm" and
                                    // "HH:mm:ss"; the field emits the former.
                                    if let Ok(time) = NaiveTime::parse_from_str(&event.value(), "%H:%M")
                                        .or_else(|_| NaiveTime::parse_from_str(&event.value(), "%H:%M:%S"))
                                    {
                                        let mut activity = activity.write();
                                        // `When.Date + TimeSpan` kept the
                                        // original kind, so a `Z` timestamp
                                        // stays `Z` here where a date edit
                                        // would not.
                                        activity.when = match activity.when {
                                            TrainerTime::Utc(naive) => {
                                                TrainerTime::Utc(naive.date().and_time(time))
                                            }
                                            TrainerTime::Offset { naive, offset } => {
                                                TrainerTime::Offset {
                                                    naive: naive.date().and_time(time),
                                                    offset,
                                                }
                                            }
                                        };
                                    }
                                },
                            }
                        }
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Amount{unit_suffix}" }
                    DecimalAmountInput {
                        // Amount is a non-null raw integer, so a cleared field
                        // maps to zero rather than to absent.
                        value: Some(current.amount),
                        decimal_places,
                        on_change: move |value: Option<i32>| {
                            activity.write().amount = value.unwrap_or(0);
                        },
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Duration (minutes or M:SS)" }
                    div { class: "input-group",
                        input {
                            r#type: "text",
                            class: "form-control",
                            placeholder: "e.g., 20 or 5:30",
                            disabled: is_active,
                            value: "{duration_input()}",
                            oninput: move |event| duration_input.set(event.value()),
                        }
                        button {
                            r#type: "button",
                            class: if is_active { "btn btn-warning" } else { "btn btn-outline-secondary" },
                            title: if is_active { "Stop timer and record duration" } else { "Start timer" },
                            onclick: move |_| async move {
                                if is_active {
                                    // Stop: write the elapsed time into the
                                    // field, save, then clear the timer.
                                    if let Some(start) = state.entries().get(&activity().id).copied() {
                                        duration_input.set(format_elapsed(now_local() - start.naive()));
                                    }
                                    if save(activity, duration_error, duration_input, selected_location_id)
                                        .await
                                    {
                                        state.finish(activity().id).await;
                                    }
                                } else if save(
                                        activity,
                                        duration_error,
                                        duration_input,
                                        selected_location_id,
                                    )
                                    .await
                                {
                                    // The activity's own When is the start, so
                                    // elapsed counts from when the activity
                                    // began rather than from the button press.
                                    state.start(activity().id, timer_start(activity().when)).await;
                                }
                            },
                            ClockIcon {}
                            {if is_active { "Stop" } else { "Start" }}
                        }
                    }
                    div { class: "form-text",
                        "Enter a whole number for minutes (e.g., "
                        code { "20" }
                        " for 20 minutes) or "
                        code { "M:SS" }
                        " for minutes and seconds (e.g., "
                        code { "5:30" }
                        " for 5 minutes 30 seconds)."
                    }
                    if let Some(error) = duration_error() {
                        div { class: "text-danger small", "{error}" }
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Notes" }
                    textarea {
                        class: "form-control",
                        rows: "4",
                        value: current.notes.clone().unwrap_or_default(),
                        oninput: move |event| activity.write().notes = Some(event.value()),
                    }
                }

                div { class: "mb-3",
                    label { class: "form-label", "Location" }
                    div { class: "input-group mb-2",
                        select {
                            class: "form-select",
                            value: selected_location_id().map(|id| id.to_string()).unwrap_or_default(),
                            onchange: move |event| {
                                selected_location_id.set(event.value().parse().ok());
                            },
                            option { value: "", "— No Location —" }
                            for location in locations.iter() {
                                option { key: "{location.id}", value: "{location.id}", "{location.name}" }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-outline-secondary",
                            onclick: move |_| {
                                navigator
                                    .push(
                                        match selected_location_id() {
                                            Some(id) => Route::KnownLocationEdit { id, return_to },
                                            None => Route::KnownLocationNew { return_to },
                                        },
                                    );
                            },
                            if selected_location_id().is_none() {
                                span { "+" }
                            } else {
                                PencilIcon {}
                            }
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-outline-secondary",
                            title: "Get Current location",
                            disabled: getting_location(),
                            onclick: move |_| async move {
                                getting_location.set(true);
                                location_error.set(None);
                                capture_location(known_locations, selected_location_id, location_error)
                                    .await;
                                getting_location.set(false);
                            },
                            if getting_location() {
                                span {
                                    class: "spinner-border spinner-border-sm",
                                    role: "status",
                                    "aria-hidden": "true",
                                }
                            } else {
                                CrosshairIcon {}
                            }
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
                            navigator.push(Route::Home {});
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}

/// Takes a fix and turns it into a selected location: an existing one when the
/// fix is near it, otherwise a new auto-named one.
///
/// Ports `ActivityEntry.GetLocationAsync`, which differs from the same button
/// on the location entry page — there a fix only fills in two number fields.
async fn capture_location(
    mut known_locations: Signal<Vec<KnownLocation>>,
    mut selected_location_id: Signal<Option<i32>>,
    mut location_error: Signal<Option<&'static str>>,
) {
    let coordinates = match current_position().await {
        Ok(coordinates) => coordinates,
        Err(GeolocationError::Denied) => return location_error.set(Some(DENIED)),
        Err(GeolocationError::Unavailable) => return location_error.set(Some(UNAVAILABLE)),
    };

    let store = storage();
    let service = KnownLocationService::new(store.inner());

    match service
        .find_nearby(coordinates.latitude, coordinates.longitude)
        .await
    {
        Ok(Some(nearby)) => selected_location_id.set(Some(nearby.id)),
        Ok(None) => {
            let Ok(name) = service.next_auto_name().await else {
                return location_error.set(Some(UNKNOWN_FAILURE));
            };
            let new_location = KnownLocation {
                id: 0,
                name,
                latitude: coordinates.latitude,
                longitude: coordinates.longitude,
            };
            let Ok(saved) = service.save(new_location).await else {
                return location_error.set(Some(UNKNOWN_FAILURE));
            };
            let mut locations = service.all().await.unwrap_or_default();
            sort_by_name(&mut locations);
            known_locations.set(locations);
            selected_location_id.set(Some(saved.id));
        }
        // The C# wrapped the whole block in a bare `catch`, which reported this
        // one message for any storage failure.
        Err(_) => location_error.set(Some(UNKNOWN_FAILURE)),
    }
}

#[component]
fn PencilIcon() -> Element {
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
            class: "me-1",
            view_box: "0 0 16 16",
            path { d: "M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z" }
            path { d: "M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z" }
        }
    }
}

#[component]
fn CrosshairIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M8 0a.5.5 0 0 1 .5.5v.518A7 7 0 0 1 14.982 7.5h.518a.5.5 0 0 1 0 1h-.518A7 7 0 0 1 8.5 14.982v.518a.5.5 0 0 1-1 0v-.518A7 7 0 0 1 1.018 8.5H.5a.5.5 0 0 1 0-1h.518A7 7 0 0 1 7.5.518V.5A.5.5 0 0 1 8 0zm0 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11zm0 3a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5z" }
        }
    }
}
