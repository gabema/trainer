//! Application routes, ported from the `@page` directives across
//! `Trainer/Pages/*.razor`.
//!
//! Nine route patterns over six pages. The three entry pages each have a create
//! form and an edit form, which Blazor expressed as two `@page` directives on
//! one component and Dioxus expresses as two variants.
//!
//! # Query parameters
//!
//! The C# read these by hand: `new Uri(Navigation.Uri)`, split the query on
//! `&`, split each pair on `=`, `Uri.UnescapeDataString` both halves, build a
//! dictionary, `TryGetValue`. That appeared three times, twice with slightly
//! different `GroupBy`/`ToDictionary` shapes. Here they are typed fields on the
//! route and the router parses them.
//!
//! Two names change as a result, because a query field is named after the Rust
//! field: `?duplicateFrom=` is now `?duplicate_from=`, and `?returnUrl=` is now
//! `?return_to=`. Both are transient in-app navigation URLs rather than
//! anything a user bookmarks. `?date=` and `?search=` on /activities are
//! unchanged, and those are the two that Calendar links to.

use crate::views::layout::MainLayout;
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Home {},

    #[route("/activities?:date&:search")]
    Activities { date: Option<String>, search: Option<String> },

    #[route("/calendar?:search")]
    Calendar { search: Option<String> },

    #[route("/activity?:duplicate_from")]
    ActivityNew { duplicate_from: Option<i32> },
    // `{Id:int}` in Blazor. A non-numeric segment fails to parse and falls
    // through to NotFound, matching Blazor's constraint behaviour.
    #[route("/activity/:id")]
    ActivityEdit { id: i32 },

    #[route("/activity-type?:return_to")]
    ActivityTypeNew { return_to: Option<i32> },
    #[route("/activity-type/:id?:return_to")]
    ActivityTypeEdit { id: i32, return_to: Option<i32> },

    #[route("/known-location?:return_to")]
    KnownLocationNew { return_to: Option<i32> },
    // A location id is a hash and can be negative. The C# could not use its
    // usual `Id > 0` test because of that and re-parsed the URL instead; here
    // the route variant already says which form this is.
    #[route("/known-location/:id?:return_to")]
    KnownLocationEdit { id: i32, return_to: Option<i32> },

    // Blazor's <NotFound> block.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// Where an entry page returns when it is done.
///
/// The C# carried a percent-encoded `returnUrl` query string that only ever
/// held `/activity` or `/activity/{id}` — the activity form is the only page
/// that links to either entry page — so the activity id alone carries it, and
/// there is no URL to parse back.
pub fn return_route(return_to: Option<i32>) -> Route {
    match return_to {
        Some(id) => Route::ActivityEdit { id },
        None => Route::ActivityNew {
            duplicate_from: None,
        },
    }
}

pub use crate::views::activities::Activities;
pub use crate::views::activity_entry::{ActivityEdit, ActivityNew};
pub use crate::views::activity_type_entry::{ActivityTypeEdit, ActivityTypeNew};
pub use crate::views::calendar::Calendar;
pub use crate::views::home::Home;
pub use crate::views::known_location_entry::{KnownLocationEdit, KnownLocationNew};

/// Ports the `<NotFound>` block from `App.razor`.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        p { role: "alert", "Sorry, there's nothing at this address." }
    }
}
