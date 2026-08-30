//! Ports `Trainer/Pages/Index.razor`.
//!
//! Three things the C# needed and this does not:
//!
//! * `Task.Delay(50)` before every chart update, and `Task.Delay(100)` after
//!   destroying the old chart, to let Blazor commit the DOM before Chart.js
//!   measured it. The chart is part of the render now, so there is nothing to
//!   wait for.
//! * `_isUpdatingChart`, a re-entrancy guard around those awaits.
//! * `_hasInitialized`, to stop `OnAfterRenderAsync` loading twice.
//!
//! The `catch { }` that swallowed every chart error also goes: there is no
//! interop call left to throw.

use crate::clock::{now_local, now_utc};
use crate::routes::Route;
use crate::state::storage;
use crate::views::activity_card::ActivityCard;
use crate::views::goal_chart::GoalChart;
use dioxus::prelude::*;
use trainer_core::helpers::search::filter_private;
use trainer_core::helpers::when::date_range;
use trainer_core::models::{Activity, ActivityType, DurationOption, KnownLocation};
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;
use trainer_core::services::export_import::ExportImportService;
use trainer_core::services::goal::chart_series;
use trainer_core::services::known_location::KnownLocationService;

/// `InputFile`'s `maxAllowedSize` in the C#.
const MAX_IMPORT_BYTES: u64 = 10 * 1024 * 1024;

/// The hidden file input the import button opens.
const IMPORT_INPUT_ID: &str = "import-file-input";

/// The four windows, in the order the button group shows them.
const DURATIONS: [(DurationOption, &str, &str); 4] = [
    (DurationOption::Last24Hours, "duration24h", "Past 24 hours"),
    (DurationOption::Week, "durationWeek", "Current Week"),
    (DurationOption::Last7Days, "duration7d", "Past 7 days"),
    (DurationOption::Last4Weeks, "duration4w", "Past 4 weeks"),
];

#[derive(Clone, Default, PartialEq)]
struct HomeData {
    activities: Vec<Activity>,
    activity_types: Vec<ActivityType>,
    known_locations: Vec<KnownLocation>,
}

async fn load_home_data() -> HomeData {
    let store = storage();
    HomeData {
        // No date range: the C# loads everything and filters in memory.
        activities: ActivityService::new(&store)
            .all(None, None)
            .await
            .unwrap_or_default(),
        activity_types: ActivityTypeService::new(store.inner())
            .all()
            .await
            .unwrap_or_default(),
        known_locations: KnownLocationService::new(store.inner())
            .all()
            .await
            .unwrap_or_default(),
    }
}

/// The result of an import or export, shown in place of the `alert()` calls the
/// C# made through `IJSRuntime`.
#[derive(Clone, PartialEq)]
struct Status {
    ok: bool,
    message: String,
}

#[component]
pub fn Home() -> Element {
    let mut reload = use_signal(|| 0u32);
    let mut selected_duration = use_signal(|| DurationOption::Week);
    let mut status = use_signal(|| None::<Status>);
    let navigator = use_navigator();

    let data = use_resource(move || {
        // Subscribes the resource to the counter, so bumping it reloads.
        let _ = reload();
        load_home_data()
    });

    let data = data.read().clone().unwrap_or_default();
    let duration = selected_duration();
    let (start, end) = date_range(duration, now_local());

    // Both the chart and the list see the same rows: inside the window, and
    // with private types removed. The C# had two near-identical methods for
    // this, one of which re-sorted a list that was already sorted.
    let in_window: Vec<Activity> = data
        .activities
        .iter()
        .filter(|a| {
            let when = a.when.naive();
            when >= start && when <= end
        })
        .cloned()
        .collect();
    let visible: Vec<Activity> = filter_private(&in_window, None, &data.activity_types)
        .into_iter()
        .cloned()
        .collect();

    let series = chart_series(
        &visible.iter().collect::<Vec<_>>(),
        &data.activity_types,
        duration,
    );

    rsx! {
        div { class: "container mt-4",
            div { class: "row mb-4",
                div { class: "col-md-12",
                    div { class: "card",
                        div { class: "card-header",
                            h5 { "Activity by Goal Duration" }
                        }
                        div { class: "card-body",
                            div { class: "mb-3",
                                div { class: "btn-group", role: "group",
                                    for (option , id , label) in DURATIONS {
                                        input {
                                            r#type: "radio",
                                            class: "btn-check",
                                            name: "duration",
                                            id,
                                            checked: duration == option,
                                            onchange: move |_| selected_duration.set(option),
                                        }
                                        label { class: "btn btn-outline-primary", r#for: id, "{label}" }
                                    }
                                }
                            }
                            GoalChart { series }
                        }
                    }
                }
            }

            crate::views::active_activities::ActiveActivities {}

            div { class: "row",
                div { class: "col-md-12",
                    div { class: "card",
                        div { class: "card-header d-flex justify-content-between align-items-center",
                            h5 {
                                class: "mb-0",
                                style: "cursor: pointer;",
                                title: "View all activities",
                                onclick: move |_| {
                                    navigator
                                        .push(Route::Activities {
                                            date: None,
                                            search: None,
                                        });
                                },
                                "Activities"
                            }
                            div { class: "header-actions",
                                button {
                                    class: "icon-button",
                                    title: "Add new Activity",
                                    onclick: move |_| {
                                        navigator.push(Route::ActivityNew { duplicate_from: None });
                                    },
                                    PlusIcon {}
                                }
                                button {
                                    class: "icon-button",
                                    title: "Export Activities",
                                    onclick: move |_| async move {
                                        status.set(Some(export().await));
                                    },
                                    ExportIcon {}
                                }
                                button {
                                    class: "icon-button",
                                    title: "Import Activities",
                                    onclick: move |_| open_import_picker(),
                                    ImportIcon {}
                                }
                            }
                        }
                        div { class: "card-body",
                            if let Some(status) = status() {
                                div {
                                    class: if status.ok { "alert alert-success" } else { "alert alert-danger" },
                                    role: "alert",
                                    "{status.message}"
                                }
                            }
                            if visible.is_empty() {
                                p { class: "text-muted",
                                    "No activities yet. Click \"Add new Activity\" to get started."
                                }
                            } else {
                                div { class: "activity-cards-container",
                                    for activity in visible {
                                        ActivityCard {
                                            key: "{activity.id}",
                                            activity: activity.clone(),
                                            activity_types: data.activity_types.clone(),
                                            known_locations: data.known_locations.clone(),
                                            on_changed: move |_| reload += 1,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // The C# reached this with JSRuntime.InvokeVoidAsync("eval", "...click()").
        input {
            id: IMPORT_INPUT_ID,
            r#type: "file",
            accept: ".json",
            class: "d-none",
            onchange: move |event| async move {
                let outcome = import(event.files()).await;
                if outcome.ok {
                    reload += 1;
                }
                status.set(Some(outcome));
            },
        }

        crate::views::layout::AppVersionFooter {}
    }
}

/// Opens the hidden file input.
///
/// The C# did this by handing `document.getElementById('fileInput').click();`
/// to `eval`. This is the same DOM call written directly, so there is no string
/// of JavaScript being compiled at runtime.
fn open_import_picker() {
    if let Some(input) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(IMPORT_INPUT_ID))
        .and_then(|el| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(el).ok())
    {
        input.click();
    }
}

async fn export() -> Status {
    let store = storage();
    // exportDate is UTC, as `DateTime.UtcNow` gave it; the file name is local,
    // as `DateTime.Now` gave it. The two disagree across midnight, which is why
    // both clocks appear here.
    match ExportImportService::new(&store).export(now_utc()).await {
        Ok(json) => {
            let file_name = format!(
                "trainer-export-{}.json",
                now_local().format("%Y%m%d-%H%M%S")
            );
            match crate::download::save_text(&file_name, &json, "application/json") {
                Ok(()) => Status {
                    ok: true,
                    message: format!("Exported to {file_name}."),
                },
                Err(error) => Status {
                    ok: false,
                    message: format!("Export failed: {error:?}"),
                },
            }
        }
        Err(error) => Status {
            ok: false,
            message: format!("Export failed: {error}"),
        },
    }
}

async fn import(files: Vec<dioxus::html::FileData>) -> Status {
    let Some(file) = files.into_iter().next() else {
        return Status {
            ok: false,
            message: "Import failed: no file selected.".to_owned(),
        };
    };

    // The C# capped the read at 10 MB through `OpenReadStream`.
    if file.size() > MAX_IMPORT_BYTES {
        return Status {
            ok: false,
            message: "Import failed: the file is larger than 10 MB.".to_owned(),
        };
    }

    let json = match file.read_string().await {
        Ok(json) => json,
        Err(error) => {
            return Status {
                ok: false,
                message: format!("Import failed: {error}"),
            };
        }
    };

    let store = storage();
    match ExportImportService::new(&store).import(&json).await {
        Ok(()) => Status {
            ok: true,
            message: "Import successful!".to_owned(),
        },
        Err(error) => Status {
            ok: false,
            message: format!("Import failed: {error}"),
        },
    }
}

#[component]
fn PlusIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z" }
        }
    }
}

/// The tray both transfer buttons sit in. Identical in the C#; the arrow is the
/// only part that differs.
#[component]
fn TrayPath() -> Element {
    rsx! {
        path { d: "M.5 9.9a.5.5 0 0 1 .5.5h2.5a.5.5 0 0 1 0 1H3a1 1 0 0 0-1 1V14a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-1.6a.5.5 0 0 1 0-1H15v-1a.5.5 0 0 1 .5-.5h.5a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-.5.5H.5a.5.5 0 0 1-.5-.5v-2a.5.5 0 0 1 .5-.5z" }
    }
}

#[component]
fn ExportIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            fill: "currentColor",
            view_box: "0 0 16 16",
            TrayPath {}
            path { d: "M7.646 11.854a.5.5 0 0 0 .708 0l3-3a.5.5 0 0 0-.708-.708L8.5 10.293V1.5a.5.5 0 0 0-1 0v8.793L5.354 8.146a.5.5 0 1 0-.708.708l3 3z" }
        }
    }
}

#[component]
fn ImportIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            fill: "currentColor",
            view_box: "0 0 16 16",
            TrayPath {}
            path { d: "M8.354 1.146a.5.5 0 0 0-.708 0l-3 3a.5.5 0 0 0 .708.708L7.5 2.707V11.5a.5.5 0 0 0 1 0V2.707l2.146 2.147a.5.5 0 0 0 .708-.708l-3-3z" }
        }
    }
}
