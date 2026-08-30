//! Ports `Trainer/Pages/Calendar.razor`.
//!
//! # The grid runs right to left
//!
//! The header reads Sat, Fri, Thu, Wed, Tue, Mon, Sun and the months descend
//! from the current one, so the whole calendar reads backwards in time from the
//! top left. That is deliberate in the C# — `GetWeeksInMonth` walks the days in
//! reverse and the render loop counts the columns down from six — and it is
//! reproduced here rather than normalized.
//!
//! # Loading
//!
//! Three months up front, one more each time the trigger scrolls into view.
//! Months are the unit on screen but weeks are the unit in storage, so a month
//! load pulls whatever week buckets it overlaps and has not already read.

use crate::clock::now_local;
use crate::routes::Route;
use crate::state::storage;
use crate::views::infinite_scroll::use_scroll_loader;
use crate::views::search_filter::SearchFilter;
use chrono::{Datelike, Months, NaiveDate, NaiveTime, Weekday};
use dioxus::prelude::*;
use futures_util::StreamExt;
use std::collections::{BTreeMap, BTreeSet};
use trainer_core::helpers::search::{filter_by_search, filter_private};
use trainer_core::models::{Activity, ActivityType, KnownLocation, NetBenefit};
use trainer_core::services::activity::ActivityService;
use trainer_core::services::activity_type::ActivityTypeService;
use trainer_core::services::known_location::KnownLocationService;
use trainer_core::week;

const TRIGGER_ID: &str = "calendar-scroll-trigger";
const INITIAL_MONTHS: u32 = 3;
/// Months are loaded past the last week with data for two years, and the
/// trigger stops firing after five. Both bounds come from the C#.
const KEEP_LOADING_MONTHS: u32 = 24;
const STOP_LOADING_MONTHS: u32 = 60;
/// Pills shown per day before collapsing into a "+N more".
const MAX_PILLS: usize = 6;
const SEARCH_PLACEHOLDER: &str = "Search by activity type, notes, amount, or location…";

/// Columns, left to right.
const COLUMNS: [Weekday; 7] = [
    Weekday::Sat,
    Weekday::Fri,
    Weekday::Thu,
    Weekday::Wed,
    Weekday::Tue,
    Weekday::Mon,
    Weekday::Sun,
];

#[derive(Clone, Copy)]
struct Page {
    months: Signal<Vec<(i32, u32)>>,
    by_day: Signal<BTreeMap<NaiveDate, Vec<Activity>>>,
    activity_types: Signal<Vec<ActivityType>>,
    known_locations: Signal<Vec<KnownLocation>>,
    loaded_weeks: Signal<BTreeSet<String>>,
    available_weeks: Signal<BTreeSet<String>>,
    oldest_month: Signal<(i32, u32)>,
    is_loading: Signal<bool>,
    has_more: Signal<bool>,
    search_term: Signal<String>,
}

/// The first of a month.
fn first_of(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, 1).expect("month is 1..=12")
}

/// The last day of a month.
fn last_of(year: i32, month: u32) -> NaiveDate {
    first_of(year, month)
        .checked_add_months(Months::new(1))
        .and_then(|d| d.pred_opt())
        .expect("a month always has a last day")
}

impl Page {
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

    /// Reads every unread week bucket the month overlaps and files the
    /// activities under their own day.
    async fn load_month(mut self, year: i32, month: u32) {
        let start = first_of(year, month).and_time(NaiveTime::MIN);
        let end = last_of(year, month).and_time(NaiveTime::MIN);

        for week_key in week::week_keys_in_range(start, end) {
            if self.loaded_weeks.peek().contains(&week_key)
                || !self.available_weeks.peek().contains(&week_key)
            {
                continue;
            }

            let (Ok(week_start), Ok(week_end)) = (
                week::week_start_date(&week_key),
                week::week_end_date(&week_key),
            ) else {
                continue;
            };

            let store = storage();
            let activities = ActivityService::new(&store)
                .all(Some(week_start.and_time(NaiveTime::MIN)), Some(week_end))
                .await
                .unwrap_or_default();

            {
                let mut by_day = self.by_day.write();
                for activity in activities {
                    by_day
                        .entry(activity.when.naive().date())
                        .or_default()
                        .push(activity);
                }
            }
            self.loaded_weeks.write().insert(week_key);
        }
    }

    async fn load_initial(mut self) {
        self.is_loading.set(true);

        let store = storage();
        let available: BTreeSet<String> = ActivityService::new(&store)
            .available_week_keys()
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();
        let any_data = !available.is_empty();
        self.available_weeks.set(available);

        let today = now_local().date();
        for back in 0..INITIAL_MONTHS {
            let Some(month) = today.checked_sub_months(Months::new(back)) else {
                break;
            };
            self.load_month(month.year(), month.month()).await;
            self.months.write().push((month.year(), month.month()));
            self.oldest_month.set((month.year(), month.month()));
        }

        self.has_more.set(any_data);
        self.is_loading.set(false);
    }

    async fn load_next_month(mut self) {
        if *self.is_loading.peek() || !*self.has_more.peek() {
            return;
        }
        self.is_loading.set(true);

        let (year, month) = *self.oldest_month.peek();
        let Some(next) = first_of(year, month).checked_sub_months(Months::new(1)) else {
            self.is_loading.set(false);
            return;
        };

        let start = next.and_time(NaiveTime::MIN);
        let end = last_of(next.year(), next.month()).and_time(NaiveTime::MIN);
        let has_data = week::week_keys_in_range(start, end).iter().any(|key| {
            self.available_weeks.peek().contains(key) && !self.loaded_weeks.peek().contains(key)
        });

        let now = now_local();
        // Empty months are still rendered for two years back, so the calendar
        // reads as a continuous run rather than jumping over gaps.
        let keep_going = now
            .checked_sub_months(Months::new(KEEP_LOADING_MONTHS))
            .is_some_and(|limit| start >= limit);
        if has_data || keep_going {
            self.load_month(next.year(), next.month()).await;
            self.months.write().push((next.year(), next.month()));
        }
        self.oldest_month.set((next.year(), next.month()));

        let unloaded = self
            .available_weeks
            .peek()
            .iter()
            .any(|key| !self.loaded_weeks.peek().contains(key));
        let within_limit = now
            .checked_sub_months(Months::new(STOP_LOADING_MONTHS))
            .is_some_and(|limit| start >= limit);
        self.has_more.set(unloaded && within_limit);

        self.is_loading.set(false);
    }

    /// Activity type ids and counts for a day, most frequent first.
    fn day_summary(&self, day: NaiveDate) -> Vec<(i32, usize)> {
        let by_day = self.by_day.read();
        let Some(activities) = by_day.get(&day) else {
            return Vec::new();
        };
        let types = self.activity_types.read();
        let locations = self.known_locations.read();
        let search = self.search_term.read();
        let term = Some(search.as_str());

        let public: Vec<Activity> = filter_private(activities, term, &types)
            .into_iter()
            .cloned()
            .collect();
        let matched = filter_by_search(&public, term, &types, &locations);

        let mut counts: BTreeMap<i32, usize> = BTreeMap::new();
        let mut order: Vec<i32> = Vec::new();
        for activity in matched {
            if !counts.contains_key(&activity.activity_type_id) {
                order.push(activity.activity_type_id);
            }
            *counts.entry(activity.activity_type_id).or_insert(0) += 1;
        }

        let mut summary: Vec<(i32, usize)> =
            order.into_iter().map(|id| (id, counts[&id])).collect();
        // `OrderByDescending` is stable, so ties keep first-appearance order.
        summary.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        summary
    }

    fn type_name(&self, activity_type_id: i32) -> String {
        self.activity_types
            .read()
            .iter()
            .find(|t| t.id == activity_type_id)
            .map_or("Unknown", |t| t.name.as_str())
            .to_owned()
    }

    fn benefit_class(&self, activity_type_id: i32) -> &'static str {
        match self
            .activity_types
            .read()
            .iter()
            .find(|t| t.id == activity_type_id)
            .map(|t| t.net_benefit)
        {
            Some(NetBenefit::Positive) => "benefit-positive",
            Some(NetBenefit::Negative) => "benefit-negative",
            _ => "benefit-neutral",
        }
    }
}

/// The weeks of a month as rows of seven day slots, indexed by weekday number
/// (0 = Sunday), newest week first.
///
/// Ports `GetWeeksInMonth`: it walks the days backwards and closes a row on
/// each Sunday, so the first row holds the end of the month and a partial row
/// at the end holds its beginning.
fn weeks_in_month(year: i32, month: u32) -> Vec<[Option<u32>; 7]> {
    let mut weeks = Vec::new();
    let mut current: [Option<u32>; 7] = [None; 7];

    for day in (1..=last_of(year, month).day()).rev() {
        let date = NaiveDate::from_ymd_opt(year, month, day).expect("day is in the month");
        let column = date.weekday().num_days_from_sunday() as usize;
        current[column] = Some(day);
        if column == 0 {
            weeks.push(current);
            current = [None; 7];
        }
    }

    if current.iter().any(Option::is_some) {
        weeks.push(current);
    }
    weeks
}

#[component]
pub fn Calendar(search: Option<String>) -> Element {
    let page = Page {
        months: use_signal(Vec::new),
        by_day: use_signal(BTreeMap::new),
        activity_types: use_signal(Vec::new),
        known_locations: use_signal(Vec::new),
        loaded_weeks: use_signal(BTreeSet::new),
        available_weeks: use_signal(BTreeSet::new),
        oldest_month: use_signal(|| {
            let today = now_local().date();
            (today.year(), today.month())
        }),
        is_loading: use_signal(|| true),
        has_more: use_signal(|| true),
        search_term: use_signal(String::new),
    };
    let navigator = use_navigator();
    let mut scroll = use_scroll_loader();

    // Unlike Activities, the search parameter only filters what is already
    // loaded, so a change to it needs no reload.
    use_effect(use_reactive!(|search| {
        let mut page = page;
        page.search_term.set(search.clone().unwrap_or_default());
    }));

    use_future(move || async move {
        page.load_types().await;
        page.load_initial().await;
    });

    use_future(move || async move {
        let Some(mut requests) = scroll.requests() else {
            return;
        };
        while requests.next().await.is_some() {
            page.load_next_month().await;
        }
    });

    let is_loading = (page.is_loading)();
    let has_more = (page.has_more)();
    let mut months = page.months.read().clone();
    months.sort_by_key(|(year, month)| std::cmp::Reverse(year * 100 + *month as i32));
    // The month name is resolved here because rsx cannot call a formatter
    // inside an interpolated attribute.
    let months: Vec<(i32, u32, String)> = months
        .into_iter()
        .map(|(year, month)| (year, month, first_of(year, month).format("%B").to_string()))
        .collect();

    use_effect(move || {
        let _ = page.months.read().len();
        let _ = page.by_day.read().len();
        if *page.has_more.read() {
            scroll.observe(TRIGGER_ID);
        }
    });

    let today = now_local().date();

    rsx! {
        div { class: "container mt-4",
            if is_loading && months.is_empty() {
                div { class: "text-center py-5",
                    div { class: "spinner-border", role: "status",
                        span { class: "visually-hidden", "Loading..." }
                    }
                }
            } else {
                div { class: "row mb-3",
                    div { class: "col-md-12",
                        div { class: "card",
                            div { class: "card-body",
                                div { class: "row g-3",
                                    div { class: "col-md-12",
                                        SearchFilter {
                                            value: page.search_term.read().clone(),
                                            placeholder: SEARCH_PLACEHOLDER.to_owned(),
                                            on_change: move |value| {
                                                let mut page = page;
                                                page.search_term.set(value);
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "calendar-container",
                    for (year , month , month_name) in months {
                        div { key: "{year}-{month}", class: "calendar-month card mb-4",
                            div { class: "card-header",
                                h5 { class: "mb-0", "{month_name} {year}" }
                            }
                            div { class: "card-body p-0",
                                div { class: "calendar-grid",
                                    div { class: "calendar-header-row",
                                        for weekday in COLUMNS {
                                            div {
                                                class: if matches!(weekday, Weekday::Sat | Weekday::Sun) { "calendar-header-cell weekend" } else { "calendar-header-cell" },
                                                {weekday.to_string()}
                                            }
                                        }
                                    }
                                    for (index , row) in weeks_in_month(year, month).into_iter().enumerate() {
                                        div { key: "{index}", class: "calendar-week-row",
                                            for weekday in COLUMNS {
                                                {
                                                    let column = weekday.num_days_from_sunday() as usize;
                                                    match row[column] {
                                                        None => rsx! {
                                                            div { class: "calendar-day-cell empty" }
                                                        },
                                                        Some(day) => {
                                                            let date = first_of(year, month).with_day(day).expect("day is in the month");
                                                            let summary = page.day_summary(date);
                                                            let overflow = summary.len().saturating_sub(MAX_PILLS);
                                                            let mut classes = String::from("calendar-day-cell");
                                                            if date == today { classes.push_str(" today"); }
                                                            if matches!(weekday, Weekday::Sat | Weekday::Sun) { classes.push_str(" weekend"); }
                                                            if !summary.is_empty() { classes.push_str(" has-activities"); }
                                                            rsx! {
                                                                div { class: "{classes}",
                                                                    div { class: "day-number", "{day}" }
                                                                    if !summary.is_empty() {
                                                                        div { class: "day-activities",
                                                                            for (type_id , count) in summary.iter().take(MAX_PILLS).copied() {
                                                                                {
                                                                                    let name = page.type_name(type_id);
                                                                                    rsx! {
                                                                                        button {
                                                                                            key: "{type_id}",
                                                                                            class: "activity-pill {page.benefit_class(type_id)}",
                                                                                            title: "{name}: {count}",
                                                                                            onclick: move |_| {
                                                                                                navigator.push(Route::Activities {
                                                                                                    date: Some(date.to_string()),
                                                                                                    search: Some(page.type_name(type_id)),
                                                                                                });
                                                                                            },
                                                                                            span { class: "activity-name", "{name}" }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            if overflow > 0 {
                                                                                button {
                                                                                    class: "activity-pill more",
                                                                                    title: "View all activities",
                                                                                    onclick: move |_| {
                                                                                        navigator.push(Route::Activities {
                                                                                            date: Some(date.to_string()),
                                                                                            search: None,
                                                                                        });
                                                                                    },
                                                                                    "+{overflow} more"
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if has_more {
                        div { id: TRIGGER_ID, style: "height: 20px;" }
                    }
                    if is_loading {
                        div { class: "text-center py-3",
                            div { class: "spinner-border spinner-border-sm", role: "status",
                                span { class: "visually-hidden", "Loading..." }
                            }
                        }
                    }
                }
            }
        }

        crate::views::layout::AppVersionFooter {}
    }
}
