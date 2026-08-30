//! Goal amounts, ported from `Trainer/Services/GoalService.cs`.

use crate::models::{Activity, ActivityType, DurationOption, NetBenefit};
use std::collections::BTreeMap;

/// The goal an activity type sets for a given filter window, or `None` when it
/// sets none.
///
/// A four-week window prefers the weekly goal times four and falls back to the
/// daily goal times twenty-eight, so a type with only a daily goal still shows
/// a target over a month.
pub fn goal_amount(activity_type: &ActivityType, duration: DurationOption) -> Option<i32> {
    match duration {
        DurationOption::Last24Hours => activity_type.daily_amount,
        DurationOption::Last7Days | DurationOption::Week => activity_type.weekly_amount,
        DurationOption::Last4Weeks => last_four_weeks_goal(activity_type),
    }
}

fn last_four_weeks_goal(activity_type: &ActivityType) -> Option<i32> {
    if let Some(weekly) = activity_type.weekly_amount {
        return Some(weekly * 4);
    }
    activity_type.daily_amount.map(|daily| daily * 28)
}

/// One activity type's progress against its goal, as the home chart plots it.
#[derive(Debug, Clone, PartialEq)]
pub struct GoalProgress {
    pub label: String,
    /// Total amount as a percentage of the goal. Unbounded above.
    pub percentage: f64,
    pub net_benefit: NetBenefit,
}

/// The chart series, ported from `Index.razor`'s `UpdateGoalDurationChart`.
///
/// Activities are grouped by type and summed, then kept only when the type
/// resolves, is not `Neutral`, and sets a positive goal for `duration`.
///
/// Ordering follows the C#: LINQ's `GroupBy` yields groups in first-appearance
/// order, and the activities arrive newest-first, so a type appears at the
/// position of its most recent activity. A `BTreeMap` keyed on type id would
/// have re-sorted the bars.
pub fn chart_series(
    activities: &[&Activity],
    activity_types: &[ActivityType],
    duration: DurationOption,
) -> Vec<GoalProgress> {
    let mut order: Vec<i32> = Vec::new();
    let mut totals: BTreeMap<i32, i32> = BTreeMap::new();

    for activity in activities {
        if !totals.contains_key(&activity.activity_type_id) {
            order.push(activity.activity_type_id);
        }
        *totals.entry(activity.activity_type_id).or_insert(0) += activity.amount;
    }

    order
        .into_iter()
        .filter_map(|type_id| {
            let activity_type = activity_types.iter().find(|t| t.id == type_id)?;
            if activity_type.net_benefit == NetBenefit::Neutral {
                return None;
            }
            // A goal of zero is skipped rather than dividing by it.
            let goal = goal_amount(activity_type, duration).filter(|g| *g > 0)?;
            Some(GoalProgress {
                label: activity_type.name.clone(),
                percentage: (totals[&type_id] as f64 / goal as f64) * 100.0,
                net_benefit: activity_type.net_benefit,
            })
        })
        .collect()
}

/// The half-range the value axis spans, matching Chart.js's `yMin`/`yMax`.
///
/// The axis is symmetric about zero and never tighter than +/-100, so a bar at
/// exactly the goal always lands on the zero line rather than at the edge.
/// Padding is a tenth of the range, floored at ten.
pub fn chart_axis_limit(series: &[GoalProgress]) -> f64 {
    let max = series
        .iter()
        .map(|p| p.percentage.abs())
        .fold(100.0_f64, f64::max);
    max + (max * 0.1).max(10.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::TrainerTime;
    use chrono::NaiveDate;

    fn activity_type(daily: Option<i32>, weekly: Option<i32>) -> ActivityType {
        ActivityType {
            id: 1,
            name: "T".to_owned(),
            net_benefit: NetBenefit::Positive,
            daily_amount: daily,
            weekly_amount: weekly,
            unit: None,
            is_private: false,
            decimal_places: 0,
        }
    }

    /// Ports `GetGoalAmount_StandardDurations_ReturnsCorrectAmount`.
    #[test]
    fn standard_durations_select_the_matching_goal() {
        let t = activity_type(Some(64), Some(420));
        assert_eq!(goal_amount(&t, DurationOption::Last24Hours), Some(64));
        assert_eq!(goal_amount(&t, DurationOption::Last7Days), Some(420));
        assert_eq!(goal_amount(&t, DurationOption::Week), Some(420));
    }

    /// Ports `GetGoalAmount_Last4Weeks_ReturnsCorrectAmount`.
    #[test]
    fn four_weeks_prefers_weekly_then_falls_back_to_daily() {
        assert_eq!(
            goal_amount(
                &activity_type(Some(10), Some(50)),
                DurationOption::Last4Weeks
            ),
            Some(200),
            "weekly wins and is multiplied by four"
        );
        assert_eq!(
            goal_amount(&activity_type(Some(10), None), DurationOption::Last4Weeks),
            Some(280),
            "daily falls back, multiplied by twenty-eight"
        );
        assert_eq!(
            goal_amount(&activity_type(None, None), DurationOption::Last4Weeks),
            None
        );
    }

    #[test]
    fn a_type_with_no_goals_has_none_for_every_window() {
        let t = activity_type(None, None);
        for duration in [
            DurationOption::Last24Hours,
            DurationOption::Last7Days,
            DurationOption::Week,
            DurationOption::Last4Weeks,
        ] {
            assert_eq!(goal_amount(&t, duration), None);
        }
    }

    fn named(id: i32, name: &str, benefit: NetBenefit, weekly: Option<i32>) -> ActivityType {
        ActivityType {
            id,
            name: name.to_owned(),
            net_benefit: benefit,
            daily_amount: None,
            weekly_amount: weekly,
            unit: None,
            is_private: false,
            decimal_places: 0,
        }
    }

    fn activity(id: i32, type_id: i32, amount: i32) -> Activity {
        Activity {
            id,
            activity_type_id: type_id,
            when: TrainerTime::Utc(
                NaiveDate::from_ymd_opt(2026, 1, 5)
                    .expect("valid date")
                    .and_hms_opt(9, 0, 0)
                    .expect("valid time"),
            ),
            amount,
            notes: None,
            duration_seconds: None,
            known_location_id: None,
        }
    }

    #[test]
    fn sums_amounts_per_type_and_divides_by_the_goal() {
        let types = [named(1, "Run", NetBenefit::Positive, Some(40))];
        let activities = [activity(1, 1, 10), activity(2, 1, 30)];
        let refs: Vec<&Activity> = activities.iter().collect();

        let series = chart_series(&refs, &types, DurationOption::Week);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].label, "Run");
        assert_eq!(series[0].percentage, 100.0);
    }

    #[test]
    fn drops_neutral_types_types_without_goals_and_unresolvable_types() {
        let types = [
            named(1, "Neutral", NetBenefit::Neutral, Some(10)),
            named(2, "No goal", NetBenefit::Positive, None),
            named(3, "Zero goal", NetBenefit::Positive, Some(0)),
        ];
        let activities = [
            activity(1, 1, 5),
            activity(2, 2, 5),
            activity(3, 3, 5),
            // Type 4 does not exist.
            activity(4, 4, 5),
        ];
        let refs: Vec<&Activity> = activities.iter().collect();

        assert!(chart_series(&refs, &types, DurationOption::Week).is_empty());
    }

    #[test]
    fn bars_follow_first_appearance_order_not_type_id() {
        let types = [
            named(1, "First id", NetBenefit::Positive, Some(10)),
            named(9, "Ninth id", NetBenefit::Positive, Some(10)),
        ];
        // Newest first, as ActivityService::all returns them.
        let activities = [activity(1, 9, 5), activity(2, 1, 5)];
        let refs: Vec<&Activity> = activities.iter().collect();

        let labels: Vec<String> = chart_series(&refs, &types, DurationOption::Week)
            .into_iter()
            .map(|p| p.label)
            .collect();
        assert_eq!(labels, ["Ninth id", "First id"]);
    }

    #[test]
    fn the_axis_never_shrinks_below_the_goal_line() {
        // Everything under goal: the axis still reaches 100 plus its padding.
        let series = [GoalProgress {
            label: "Run".to_owned(),
            percentage: 12.0,
            net_benefit: NetBenefit::Positive,
        }];
        assert_eq!(chart_axis_limit(&series), 110.0);
    }

    #[test]
    fn the_axis_grows_with_the_largest_bar_in_either_direction() {
        let series = [GoalProgress {
            label: "Snacks".to_owned(),
            percentage: 300.0,
            net_benefit: NetBenefit::Negative,
        }];
        assert_eq!(chart_axis_limit(&series), 330.0);
    }

    #[test]
    fn an_empty_series_still_yields_the_minimum_axis() {
        assert_eq!(chart_axis_limit(&[]), 110.0);
    }
}
