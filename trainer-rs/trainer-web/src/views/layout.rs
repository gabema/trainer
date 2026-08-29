//! The application shell, ported from `Trainer/Layout/`.
//!
//! `NavMenu.razor` and `NavMenu.razor.css` are **not** ported: 112 lines that
//! nothing rendered. `MainLayout` uses `TopNavBar`, and `NavMenu` appears
//! nowhere else in the C# source — it is the Blazor template's default nav,
//! replaced and never deleted.

use crate::build_info::VERSION;
use crate::routes::Route;
use dioxus::prelude::*;

/// Ports `MainLayout.razor`. `Outlet` takes the place of `@Body`.
#[component]
pub fn MainLayout() -> Element {
    rsx! {
        div { class: "page",
            TopNavBar {}
            main {
                article { class: "content px-4",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

/// Ports `TopNavBar.razor`.
///
/// The C# computed the active tab by hand: it read `NavigationManager.Uri`,
/// stripped the base URI, and compared prefixes — then subscribed to
/// `LocationChanged`, called `StateHasChanged`, and implemented `IDisposable`
/// purely to re-render on navigation.
///
/// `Link`'s `active_class` does all of that declaratively, so the subscription,
/// the manual redraw and the disposal all disappear.
#[component]
pub fn TopNavBar() -> Element {
    rsx! {
        nav { class: "top-nav-bar",
            div { class: "nav-tabs-container",
                Link {
                    to: Route::Home {},
                    class: "nav-tab",
                    active_class: "active",
                    HomeIcon {}
                    span { "Home" }
                }
                Link {
                    to: Route::Activities {},
                    class: "nav-tab",
                    active_class: "active",
                    ActivitiesIcon {}
                    span { "Activities" }
                }
                Link {
                    to: Route::Calendar {},
                    class: "nav-tab",
                    active_class: "active",
                    CalendarIcon {}
                    span { "Calendar" }
                }
            }
        }
    }
}

/// Ports `AppVersionFooter.razor`.
#[component]
pub fn AppVersionFooter() -> Element {
    rsx! {
        footer { class: "text-muted small text-center py-2", "{VERSION}" }
    }
}

#[component]
fn HomeIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "20",
            height: "20",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M6.5 14.5v-3.505c0-.245.25-.495.5-.495h2c.25 0 .5.25.5.5v3.5a.5.5 0 0 0 .5.5h4a.5.5 0 0 0 .5-.5v-7a.5.5 0 0 0-.146-.354L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293L8.354 1.146a.5.5 0 0 0-.708 0l-6 6A.5.5 0 0 0 1.5 7.5v7a.5.5 0 0 0 .5.5h4a.5.5 0 0 0 .5-.5Z" }
        }
    }
}

#[component]
fn ActivitiesIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "20",
            height: "20",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path {
                fill_rule: "evenodd",
                d: "M5 11.5a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m0-4a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m0-4a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5M3.854 2.146a.5.5 0 0 1 0 .708l-1.5 1.5a.5.5 0 0 1-.708 0l-.5-.5a.5.5 0 1 1 .708-.708L2 3.293l1.146-1.147a.5.5 0 0 1 .708 0m0 4a.5.5 0 0 1 0 .708l-1.5 1.5a.5.5 0 0 1-.708 0l-.5-.5a.5.5 0 1 1 .708-.708L2 7.293l1.146-1.147a.5.5 0 0 1 .708 0m0 4a.5.5 0 0 1 0 .708l-1.5 1.5a.5.5 0 0 1-.708 0l-.5-.5a.5.5 0 0 1 .708-.708l.146.147 1.146-1.147a.5.5 0 0 1 .708 0"
            }
        }
    }
}

#[component]
fn CalendarIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "20",
            height: "20",
            fill: "currentColor",
            view_box: "0 0 16 16",
            path { d: "M3.5 0a.5.5 0 0 1 .5.5V1h8V.5a.5.5 0 0 1 1 0V1h1a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V3a2 2 0 0 1 2-2h1V.5a.5.5 0 0 1 .5-.5M1 4v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V4z" }
        }
    }
}
