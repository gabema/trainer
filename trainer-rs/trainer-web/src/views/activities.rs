//! Ports `Trainer/Pages/Activities.razor`.
//!
//! # How lazy loading works
//!
//! Activities are stored one bucket per ISO-ish week key, so the page loads
//! whole weeks, newest first: eight up front, then one more each time the
//! trigger element scrolls into view. Filtering happens client-side over what
//! has been loaded.
//!
//! That combination is what issue #85 is about. Under All time with a search
//! term, the filter can leave the page too short to scroll, so the observer
//! never fires again and matches in older weeks never surface. `week_fill::fill`
//! keeps loading older weeks until enough matches are displayed, and the
//! trigger is rendered whenever weeks remain — even when nothing matches yet.
//!
//! # What the port drops
//!
//! `OnParametersSetAsync` hand-tracked `_lastFilterDate` and `_lastFilterSearch`
//! to work out whether the query string had changed, and `_initialized` guarded
//! against `OnAfterRenderAsync` running twice. An effect keyed on the route's
//! own parameters does both.

use crate::clock::now_local;
use crate::routes::Route;
use crate::state::storage;
use crate::views::activity_card::ActivityCard;
use crate::views::infinite_scroll::use_scroll_loader;
use crate::views::search_filter::SearchFilter;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;
use futures_util::StreamExt;
use std::collections::BTreeSet;
use trainer_core::helpers::search::{filter_by_search, filter_private};
use trainer_core::models::{Activity, ActivityType, KnownLocation};
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;
use trainer_core::services::known_location::KnownLocationService;
use trainer_core::services::week_fill;
use trainer_core::week;

const TRIGGER_ID: &str = "scroll-trigger";
const MAX_INITIAL_WEEKS: usize = 8;
/// Minimum matches to keep loading toward under All time with a search term,
/// so the page becomes scrollable and older matches surface (issue #85).
const MIN_DISPLAYED_TO_FILL: usize = 20;
const SEARCH_PLACEHOLDER: &str = "Search by activity type, notes, amount, or location…";

#[derive(Clone, Copy, PartialEq, Eq)]
enum DateFilter {
    AllTime,
    Custom,
}

/// Everything the page loads and filters on. Bundled so the loading routines
/// can be free functions rather than a wall of parameters.
#[derive(Clone, Copy)]
struct Page {
    all_loaded: Signal<Vec<Activity>>,
    loaded_weeks: Signal<BTreeSet<String>>,
    available_weeks: Signal<Vec<String>>,
    activity_types: Signal<Vec<ActivityType>>,
    known_locations: Signal<Vec<KnownLocation>>,
    search_term: Signal<String>,
    date_filter: Signal<DateFilter>,
    custom_start: Signal<Option<NaiveDate>>,
    custom_end: Signal<Option<NaiveDate>>,
    is_loading: Signal<bool>,
    has_more: Signal<bool>,
}

/// End of day, as `GetDateFilterRange` built it.
fn end_of_day(date: NaiveDate) -> NaiveDateTime {
    date.and_time(NaiveTime::from_hms_opt(23, 59, 59).expect("23:59:59 is a valid time"))
}

/// The filter chain, with no signal access of its own.
fn visible_rows(
    all: &[Activity],
    types: &[ActivityType],
    locations: &[KnownLocation],
    search: &str,
    range: Option<(NaiveDateTime, NaiveDateTime)>,
) -> Vec<Activity> {
    let term = Some(search);

    let in_window: Vec<Activity> = match range {
        Some((start, end)) => all
            .iter()
            .filter(|a| {
                let when = a.when.naive();
                when >= start && when <= end
            })
            .cloned()
            .collect(),
        None => all.to_vec(),
    };

    let public: Vec<Activity> = filter_private(&in_window, term, types)
        .into_iter()
        .cloned()
        .collect();
    let mut matched: Vec<Activity> = filter_by_search(&public, term, types, locations)
        .into_iter()
        .cloned()
        .collect();
    matched.sort_by_key(|a| std::cmp::Reverse(a.when.naive()));
    matched
}

impl Page {
    /// The active window, or `None` under All time or a half-filled custom
    /// range.
    fn range(&self) -> Option<(NaiveDateTime, NaiveDateTime)> {
        if *self.date_filter.peek() != DateFilter::Custom {
            return None;
        }
        let start = (*self.custom_start.peek())?;
        let end = (*self.custom_end.peek())?;
        Some((start.and_time(NaiveTime::MIN), end_of_day(end)))
    }

    /// The rows the list shows: inside the window, private types removed unless
    /// searched for by name, then matched against the search term, newest
    /// first.
    ///
    /// Reads reactively, so calling it from the render body subscribes the
    /// component to everything it depends on.
    fn displayed(&self) -> Vec<Activity> {
        visible_rows(
            &self.all_loaded.read(),
            &self.activity_types.read(),
            &self.known_locations.read(),
            &self.search_term.read(),
            self.range(),
        )
    }

    /// The same count, read without subscribing.
    ///
    /// The fill loop runs inside a spawned task, and a reactive read there
    /// could attach a subscription to whatever context happened to start it —
    /// which, since the loop also writes `all_loaded`, would be a cycle.
    fn displayed_count(&self) -> usize {
        visible_rows(
            &self.all_loaded.peek(),
            &self.activity_types.peek(),
            &self.known_locations.peek(),
            &self.search_term.peek(),
            self.range(),
        )
        .len()
    }

    /// Reads a week's bucket. `ActivityService::all` resolves the range back to
    /// week keys, which is how the C# fetched a single week too.
    async fn fetch_week(week_key: &str) -> Vec<Activity> {
        let (Ok(start), Ok(end)) = (
            week::week_start_date(week_key),
            week::week_end_date(week_key),
        ) else {
            return Vec::new();
        };
        let store = storage();
        ActivityService::new(&store)
            .all(Some(start.and_time(NaiveTime::MIN)), Some(end))
            .await
            .unwrap_or_default()
    }

    async fn append_week(mut self, week_key: &str) {
        let activities = Self::fetch_week(week_key).await;
        self.all_loaded.write().extend(activities);
    }

    async fn load_types(mut self) {
        let store = storage();
        self.activity_types.set(
            ActivityTypeService::new(store.inner())
                .all()
                .await
                .unwrap_or_default(),
        );
        self.known_locations.set(
            KnownLocationService::new(store.inner())
                .all()
                .await
                .unwrap_or_default(),
        );
    }

    /// Clears everything loaded and loads the first batch again. Every filter
    /// change and every delete goes through here, as in the C#.
    async fn reload(mut self) {
        self.all_loaded.write().clear();
        self.loaded_weeks.write().clear();
        self.available_weeks.write().clear();
        self.has_more.set(true);
        self.load_initial().await;
    }

    async fn load_initial(mut self) {
        self.is_loading.set(true);

        let store = storage();
        let available = ActivityService::new(&store)
            .available_week_keys()
            .await
            .unwrap_or_default();
        self.available_weeks.set(available.clone());

        match self.range() {
            Some((start, end)) => {
                // A date filter loads every week the range touches that has
                // data, so there is nothing left to lazily load afterwards.
                let in_range: BTreeSet<String> =
                    week::week_keys_in_range(start, end).into_iter().collect();
                let mut to_load: Vec<String> = available
                    .iter()
                    .filter(|key| in_range.contains(*key))
                    .cloned()
                    .collect();
                to_load.sort_by(|a, b| b.cmp(a));

                for week_key in to_load {
                    if self.loaded_weeks.peek().contains(&week_key) {
                        continue;
                    }
                    self.append_week(&week_key).await;
                    self.loaded_weeks.write().insert(week_key);
                }
                self.has_more.set(false);
            }
            None => {
                let mut sorted = available.clone();
                sorted.sort_by(|a, b| b.cmp(a));

                for week_key in sorted.into_iter().take(MAX_INITIAL_WEEKS) {
                    if self.loaded_weeks.peek().contains(&week_key) {
                        continue;
                    }
                    self.append_week(&week_key).await;
                    self.loaded_weeks.write().insert(week_key);
                }
                let loaded = self.loaded_weeks.peek().len();
                self.has_more.set(loaded < available.len());
                self.fill_for_search().await;
            }
        }

        self.is_loading.set(false);
    }

    async fn load_next_week(mut self) {
        if *self.is_loading.peek() || !*self.has_more.peek() {
            return;
        }
        self.is_loading.set(true);
        self.load_one_more_week().await;
        self.fill_for_search().await;
        self.is_loading.set(false);
    }

    async fn load_one_more_week(mut self) {
        let available = self.available_weeks.peek().clone();
        let loaded = self.loaded_weeks.peek().clone();

        let next = match self.range() {
            Some((start, end)) => {
                let in_range: BTreeSet<String> =
                    week::week_keys_in_range(start, end).into_iter().collect();
                available
                    .iter()
                    .filter(|key| in_range.contains(*key) && !loaded.contains(*key))
                    .max()
                    .cloned()
            }
            None => week_fill::next_week_key(&available, &loaded),
        };

        let Some(next) = next else {
            self.has_more.set(false);
            return;
        };

        self.append_week(&next).await;
        self.loaded_weeks.write().insert(next);

        let loaded = self.loaded_weeks.peek().clone();
        let remaining = match self.range() {
            Some((start, end)) => {
                let in_range: BTreeSet<String> =
                    week::week_keys_in_range(start, end).into_iter().collect();
                available
                    .iter()
                    .any(|key| in_range.contains(key) && !loaded.contains(key))
            }
            None => available.iter().any(|key| !loaded.contains(key)),
        };
        self.has_more.set(remaining);
    }

    /// Issue #85: under All time with a search term, keep loading older weeks
    /// until enough matches show to make the page scrollable.
    async fn fill_for_search(mut self) {
        if *self.date_filter.peek() != DateFilter::AllTime
            || self.search_term.peek().trim().is_empty()
        {
            return;
        }

        let available = self.available_weeks.peek().clone();
        let mut loaded = self.loaded_weeks.peek().clone();

        let remaining = week_fill::fill(
            &available,
            &mut loaded,
            |week_key, loaded| {
                // The set is recorded synchronously — the borrow cannot cross
                // the await — and the fetch happens in the returned future.
                loaded.insert(week_key.clone());
                async move { self.append_week(&week_key).await }
            },
            || self.displayed_count(),
            MIN_DISPLAYED_TO_FILL,
        )
        .await;

        self.loaded_weeks.set(loaded);
        self.has_more.set(remaining);
    }
}

#[component]
pub fn Activities(date: Option<String>, search: Option<String>) -> Element {
    let page = Page {
        all_loaded: use_signal(Vec::new),
        loaded_weeks: use_signal(BTreeSet::new),
        available_weeks: use_signal(Vec::new),
        activity_types: use_signal(Vec::new),
        known_locations: use_signal(Vec::new),
        search_term: use_signal(String::new),
        date_filter: use_signal(|| DateFilter::AllTime),
        custom_start: use_signal(|| None),
        custom_end: use_signal(|| None),
        is_loading: use_signal(|| false),
        has_more: use_signal(|| true),
    };
    let navigator = use_navigator();
    let mut scroll = use_scroll_loader();
    let mut last_params = use_signal(|| (None::<String>, None::<String>));

    // Query parameters drive the filters, and a change to either resets and
    // reloads. This also covers the first render, which is why there is no
    // separate initial-load path.
    use_effect(use_reactive!(|(date, search)| {
        // `peek`, not a reactive read: this effect writes `last_params`, and
        // subscribing to what it writes would re-trigger it forever.
        let (old_date, old_search) = last_params.peek().clone();
        last_params.set((date.clone(), search.clone()));

        let mut page = page;
        // A cleared parameter resets the filter only if it had previously been
        // set, so navigating to a plain /activities does not wipe what the user
        // typed into the search box.
        match search.as_deref().filter(|s| !s.is_empty()) {
            Some(term) => page.search_term.set(term.to_owned()),
            None if old_search.is_some_and(|s| !s.is_empty()) => {
                page.search_term.set(String::new());
            }
            None => {}
        }
        match date
            .as_deref()
            .filter(|d| !d.is_empty())
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        {
            Some(day) => {
                page.date_filter.set(DateFilter::Custom);
                page.custom_start.set(Some(day));
                page.custom_end.set(Some(day));
            }
            None if old_date.is_some_and(|d| !d.is_empty()) => {
                page.date_filter.set(DateFilter::AllTime);
                page.custom_start.set(None);
                page.custom_end.set(None);
            }
            None => {}
        }

        spawn(async move {
            if page.activity_types.peek().is_empty() {
                page.load_types().await;
            }
            page.reload().await;
        });
    }));

    // Drains the trigger's notifications. Everything the browser-side callback
    // does is a channel send; the loading happens here, inside the runtime.
    use_future(move || async move {
        let Some(mut requests) = scroll.requests() else {
            return;
        };
        while requests.next().await.is_some() {
            page.load_next_week().await;
        }
    });

    let displayed = page.displayed();
    let is_loading = (page.is_loading)();
    let has_more = (page.has_more)();

    // Re-arm after every render that changes the list or the trigger's
    // presence: a trigger that never leaves the viewport must keep firing.
    use_effect(move || {
        let _ = page.all_loaded.read().len();
        if *page.has_more.read() {
            scroll.observe(TRIGGER_ID);
        }
    });

    let filtering = !page.search_term.read().trim().is_empty()
        || *page.date_filter.read() != DateFilter::AllTime;

    rsx! {
        div { class: "container mt-4",
            div { class: "row mb-3",
                div { class: "col-md-12",
                    div { class: "card",
                        div { class: "card-body",
                            div { class: "row g-3",
                                div { class: "col-md-9",
                                    SearchFilter {
                                        value: page.search_term.read().clone(),
                                        placeholder: SEARCH_PLACEHOLDER.to_owned(),
                                        on_change: move |value| {
                                            let mut page = page;
                                            page.search_term.set(value);
                                        },
                                    }
                                }
                                div { class: "col-md-3",
                                    label { class: "form-label", "Date Duration" }
                                    select {
                                        class: "form-select",
                                        value: if *page.date_filter.read() == DateFilter::Custom { "1" } else { "0" },
                                        onchange: move |event| async move {
                                            let mut page = page;
                                            let custom = event.value() == "1";
                                            page.date_filter
                                                .set(if custom { DateFilter::Custom } else { DateFilter::AllTime });
                                            if custom {
                                                // The C#'s default custom range.
                                                let today = now_local().date();
                                                page.custom_end.set(Some(today));
                                                page.custom_start
                                                    .set(today.checked_sub_days(chrono::Days::new(7)));
                                            } else {
                                                page.custom_start.set(None);
                                                page.custom_end.set(None);
                                            }
                                            page.reload().await;
                                        },
                                        option { value: "0", "All time" }
                                        option { value: "1", "Custom Range" }
                                    }
                                }
                            }
                            if *page.date_filter.read() == DateFilter::Custom {
                                div { class: "row g-3 mt-2",
                                    div { class: "col-md-6",
                                        label { class: "form-label", "Start Date" }
                                        input {
                                            r#type: "date",
                                            class: "form-control",
                                            value: page.custom_start.read().map_or_else(String::new, |d| d.to_string()),
                                            onchange: move |event| async move {
                                                let mut page = page;
                                                if let Ok(date) = NaiveDate::parse_from_str(&event.value(), "%Y-%m-%d") {
                                                    page.custom_start.set(Some(date));
                                                    if page.custom_end.peek().is_some() {
                                                        page.reload().await;
                                                    }
                                                }
                                            },
                                        }
                                    }
                                    div { class: "col-md-6",
                                        label { class: "form-label", "End Date" }
                                        input {
                                            r#type: "date",
                                            class: "form-control",
                                            value: page.custom_end.read().map_or_else(String::new, |d| d.to_string()),
                                            onchange: move |event| async move {
                                                let mut page = page;
                                                if let Ok(date) = NaiveDate::parse_from_str(&event.value(), "%Y-%m-%d") {
                                                    page.custom_end.set(Some(date));
                                                    if page.custom_start.peek().is_some() {
                                                        page.reload().await;
                                                    }
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                            div { class: "mt-3",
                                button {
                                    class: "btn btn-primary",
                                    onclick: move |_| {
                                        navigator.push(Route::ActivityNew { duplicate_from: None });
                                    },
                                    PlusIcon {}
                                    "Add New Activity"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "row",
                div { class: "col-md-12",
                    div { class: "card",
                        div { class: "card-body",
                            if !displayed.is_empty() {
                                div { class: "activity-cards-container",
                                    for activity in displayed {
                                        ActivityCard {
                                            key: "{activity.id}",
                                            activity: activity.clone(),
                                            activity_types: page.activity_types.read().clone(),
                                            known_locations: page.known_locations.read().clone(),
                                            on_changed: move |_| {
                                                spawn(async move { page.reload().await });
                                            },
                                        }
                                    }
                                }
                            } else if !is_loading && !has_more {
                                p { class: "text-muted",
                                    "No activities found. "
                                    if filtering {
                                        "Try adjusting your search or filters."
                                    } else {
                                        "Click \"Add New Activity\" to get started."
                                    }
                                }
                            }

                            // Rendered whenever weeks remain, even with nothing
                            // matching yet, so loading can continue through
                            // weeks that hold no matches (issue #85).
                            if has_more {
                                div { id: TRIGGER_ID, style: "height: 20px;" }
                            }
                            if is_loading {
                                div { class: "text-center py-3",
                                    div { class: "spinner-border", role: "status",
                                        span { class: "visually-hidden", "Loading..." }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        crate::views::layout::AppVersionFooter {}
    }
}

#[component]
fn PlusIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            fill: "currentColor",
            class: "me-1",
            view_box: "0 0 16 16",
            path { d: "M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z" }
        }
    }
}
