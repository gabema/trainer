//! Ports `Trainer/Components/SearchFilter.razor`.
//!
//! The C# was a two-way `Value` / `ValueChanged` pair; here the value is a
//! plain prop and changes go back through one `EventHandler`, which is the same
//! contract without the naming convention.

use dioxus::prelude::*;

/// The placeholder `SearchFilter.razor` declared as its parameter default.
/// Both callers override it, but the default is part of the component.
const DEFAULT_PLACEHOLDER: &str = "Search by activity type, notes, or amount...";

#[component]
pub fn SearchFilter(
    value: String,
    on_change: EventHandler<String>,
    #[props(default = DEFAULT_PLACEHOLDER.to_owned())] placeholder: String,
) -> Element {
    rsx! {
        label { class: "form-label", "Search" }
        input {
            r#type: "text",
            class: "form-control",
            value: "{value}",
            placeholder: "{placeholder}",
            oninput: move |event| on_change.call(event.value()),
        }
    }
}
