//! Activity summary text, ported from `Trainer/Helpers/ActivityAmountDisplay.cs`.

use super::amount;
use crate::models::{Activity, ActivityType, KnownLocation};

/// Compact duration text: `45s`, `10m`, `5m 5s`.
///
/// Single-digit seconds are deliberately **not** zero-padded — `5m 5s`, never
/// `5m 05s`, per the `activity-duration` capability. A missing or non-positive
/// duration renders nothing.
pub fn format_duration(duration_seconds: Option<i32>) -> Option<String> {
    let total = duration_seconds?;
    if total <= 0 {
        return None;
    }

    let minutes = total / 60;
    let seconds = total % 60;

    Some(match (minutes, seconds) {
        (0, s) => format!("{s}s"),
        (m, 0) => format!("{m}m"),
        (m, s) => format!("{m}m {s}s"),
    })
}

/// The one-line summary shown on an activity card:
/// `5 km for 30m @ Gym`.
///
/// Each clause is omitted when its source is absent, and an unresolvable
/// activity type or location is treated as absent rather than an error — a
/// dangling `known_location_id` simply drops the location suffix.
pub fn format_activity(
    activity: &Activity,
    activity_types: &[ActivityType],
    known_locations: &[KnownLocation],
) -> String {
    let activity_type = activity_types
        .iter()
        .find(|t| t.id == activity.activity_type_id);

    let decimal_places = activity_type.map_or(0, |t| t.decimal_places);
    let amount_text = amount::format_display(activity.amount, decimal_places);

    let mut result = match activity_type.and_then(|t| t.unit.as_deref()) {
        Some(unit) => format!("{amount_text} {unit}"),
        None => amount_text,
    };

    if let Some(duration) = format_duration(activity.duration_seconds) {
        result = format!("{result} for {duration}");
    }

    if let Some(location) = activity
        .known_location_id
        .and_then(|id| known_locations.iter().find(|l| l.id == id))
    {
        result = format!("{result} @ {}", location.name);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::TrainerTime;
    use crate::models::NetBenefit;

    fn run_type() -> ActivityType {
        ActivityType {
            id: 1,
            name: "Running".to_owned(),
            net_benefit: NetBenefit::Neutral,
            daily_amount: None,
            weekly_amount: None,
            unit: Some("km".to_owned()),
            is_private: false,
            decimal_places: 0,
        }
    }

    fn no_unit_type() -> ActivityType {
        ActivityType {
            unit: None,
            id: 2,
            name: "Pushups".to_owned(),
            ..run_type()
        }
    }

    fn gym() -> KnownLocation {
        KnownLocation {
            id: 10,
            name: "Gym".to_owned(),
            latitude: 0.0,
            longitude: 0.0,
        }
    }

    fn make(
        activity_type_id: i32,
        amount: i32,
        duration_seconds: Option<i32>,
        known_location_id: Option<i32>,
    ) -> Activity {
        Activity {
            id: 1,
            activity_type_id,
            when: TrainerTime::parse("2026-01-01T08:00:00Z").expect("parses"),
            amount,
            notes: None,
            duration_seconds,
            known_location_id,
        }
    }

    // ── Amount formatting ─────────────────────────────────────────────────

    #[test]
    fn amount_only_when_there_is_no_unit_duration_or_location() {
        let result = format_activity(&make(2, 10, None, None), &[no_unit_type()], &[]);
        assert_eq!(result, "10");
    }

    #[test]
    fn amount_with_unit() {
        let result = format_activity(&make(1, 5, None, None), &[run_type()], &[]);
        assert_eq!(result, "5 km");
    }

    // ── Duration formatting ───────────────────────────────────────────────

    #[test]
    fn amount_with_whole_minute_duration() {
        let result = format_activity(&make(1, 5, Some(1200), None), &[run_type()], &[]);
        assert_eq!(result, "5 km for 20m");
    }

    #[test]
    fn amount_with_minutes_and_seconds_duration() {
        let result = format_activity(&make(1, 5, Some(330), None), &[run_type()], &[]);
        assert_eq!(result, "5 km for 5m 30s");
    }

    #[test]
    fn amount_with_unit_and_duration() {
        let result = format_activity(&make(1, 10, Some(600), None), &[run_type()], &[]);
        assert_eq!(result, "10 km for 10m");
    }

    // ── Location formatting ───────────────────────────────────────────────

    #[test]
    fn amount_with_unit_duration_and_location() {
        let result = format_activity(&make(1, 5, Some(1800), Some(10)), &[run_type()], &[gym()]);
        assert_eq!(result, "5 km for 30m @ Gym");
    }

    #[test]
    fn amount_with_unit_and_location_but_no_duration() {
        let result = format_activity(&make(1, 5, None, Some(10)), &[run_type()], &[gym()]);
        assert_eq!(result, "5 km @ Gym");
    }

    #[test]
    fn unmatched_location_id_omits_the_suffix() {
        let result = format_activity(&make(1, 5, None, Some(99)), &[run_type()], &[gym()]);
        assert_eq!(result, "5 km");
    }

    #[test]
    fn empty_location_list_omits_the_suffix() {
        let result = format_activity(&make(1, 5, None, Some(10)), &[run_type()], &[]);
        assert_eq!(result, "5 km");
    }

    #[test]
    fn unmatched_activity_type_falls_back_to_zero_decimal_places_and_no_unit() {
        let result = format_activity(&make(42, 7, None, None), &[run_type()], &[]);
        assert_eq!(result, "7");
    }

    #[test]
    fn decimal_places_are_applied_from_the_activity_type() {
        let mut water = run_type();
        water.decimal_places = 2;
        water.unit = Some("L".to_owned());
        let result = format_activity(&make(1, 125, None, None), &[water], &[]);
        assert_eq!(result, "1.25 L");
    }

    // ── FormatDuration edge cases ─────────────────────────────────────────

    #[test]
    fn duration_under_a_minute_shows_seconds_only() {
        assert_eq!(format_duration(Some(45)).as_deref(), Some("45s"));
    }

    #[test]
    fn single_digit_seconds_are_not_zero_padded() {
        for (seconds, expected) in [(305, "5m 5s"), (330, "5m 30s"), (600, "10m")] {
            assert_eq!(format_duration(Some(seconds)).as_deref(), Some(expected));
        }
    }

    #[test]
    fn absent_or_non_positive_duration_renders_nothing() {
        assert_eq!(format_duration(None), None);
        assert_eq!(format_duration(Some(0)), None);
        assert_eq!(format_duration(Some(-30)), None);
    }
}
