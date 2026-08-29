//! Goal amounts, ported from `Trainer/Services/GoalService.cs`.

use crate::models::{ActivityType, DurationOption};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NetBenefit;

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
}
