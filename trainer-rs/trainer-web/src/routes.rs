//! Application routes, ported from the `@page` directives across
//! `Trainer/Pages/*.razor`.
//!
//! Nine route patterns over six pages. The three entry pages each have a
//! create form and an edit form, which Blazor expressed as two `@page`
//! directives on one component and Dioxus expresses as two variants.

use crate::views::layout::MainLayout;
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Home {},

    #[route("/activities")]
    Activities {},

    #[route("/calendar")]
    Calendar {},

    #[route("/activity")]
    ActivityNew {},
    // `{Id:int}` in Blazor. A non-numeric segment fails to parse and falls
    // through to NotFound, matching Blazor's constraint behaviour.
    #[route("/activity/:id")]
    ActivityEdit { id: i32 },

    #[route("/activity-type")]
    ActivityTypeNew {},
    #[route("/activity-type/:id")]
    ActivityTypeEdit { id: i32 },

    #[route("/known-location")]
    KnownLocationNew {},
    #[route("/known-location/:id")]
    KnownLocationEdit { id: i32 },

    // Blazor's <NotFound> block.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// Placeholder pages. Each is replaced in section 5; they exist now so routing
/// can be verified before any page is ported.
macro_rules! placeholder {
    ($name:ident, $title:literal) => {
        #[component]
        pub fn $name() -> Element {
            rsx! {
                h1 { class: "h4 mb-3", $title }
                p { class: "text-muted", "Not ported yet." }
            }
        }
    };
    ($name:ident, $title:literal, id) => {
        #[component]
        pub fn $name(id: i32) -> Element {
            rsx! {
                h1 { class: "h4 mb-3", $title }
                p { class: "text-muted", "Not ported yet. id = {id}" }
            }
        }
    };
}

// Home, Activities and Calendar carry the version footer. The three entry
// pages deliberately do not, matching the app-version-footer spec, which is why
// the footer lives on the pages rather than in MainLayout.
macro_rules! placeholder_with_footer {
    ($name:ident, $title:literal) => {
        #[component]
        pub fn $name() -> Element {
            rsx! {
                h1 { class: "h4 mb-3", $title }
                p { class: "text-muted", "Not ported yet." }
                crate::views::layout::AppVersionFooter {}
            }
        }
    };
}

placeholder_with_footer!(Home, "Home");
placeholder_with_footer!(Activities, "Activities");
placeholder_with_footer!(Calendar, "Calendar");
placeholder!(ActivityNew, "Add Activity");
placeholder!(ActivityEdit, "Edit Activity", id);
placeholder!(ActivityTypeNew, "Add Activity Type");
placeholder!(ActivityTypeEdit, "Edit Activity Type", id);
placeholder!(KnownLocationNew, "Add Known Location");
placeholder!(KnownLocationEdit, "Edit Known Location", id);

/// Ports the `<NotFound>` block from `App.razor`.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        p { role: "alert", "Sorry, there's nothing at this address." }
    }
}
