//! Activity filtering, ported from `Trainer/Helpers/ActivitySearchFilter.cs`.

use super::amount;
use crate::models::{Activity, ActivityType, KnownLocation};

/// Case-insensitive substring test.
///
/// The C# uses `StringComparison.OrdinalIgnoreCase`, which folds case per code
/// unit. Rust's `to_lowercase` applies full Unicode lowercasing, which can
/// change length for a handful of characters. The two agree across ASCII and
/// everything realistic here; the difference is noted rather than worked around.
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn is_blank(term: Option<&str>) -> bool {
    term.is_none_or(|t| t.trim().is_empty())
}

fn matches_search(
    activity: &Activity,
    term: &str,
    activity_types: &[ActivityType],
    known_locations: &[KnownLocation],
) -> bool {
    let activity_type = activity_types
        .iter()
        .find(|t| t.id == activity.activity_type_id);

    let type_name = activity_type.map_or("", |t| t.name.as_str());

    let location_name = activity
        .known_location_id
        .and_then(|id| known_locations.iter().find(|l| l.id == id))
        .map_or("", |l| l.name.as_str());

    // Search matches the *displayed* amount, so a type with 2 decimal places
    // matches "1.25" rather than the stored "125".
    let amount_text = amount::format_display(
        activity.amount,
        activity_type.map_or(0, |t| t.decimal_places),
    );

    contains_ignore_case(type_name, term)
        || contains_ignore_case(activity.notes.as_deref().unwrap_or(""), term)
        || contains_ignore_case(&amount_text, term)
        || contains_ignore_case(location_name, term)
}

/// Filters by search term across activity type name, notes, displayed amount,
/// and known-location name. A blank term returns everything unchanged.
pub fn filter_by_search<'a>(
    activities: &'a [Activity],
    search_term: Option<&str>,
    activity_types: &[ActivityType],
    known_locations: &[KnownLocation],
) -> Vec<&'a Activity> {
    if is_blank(search_term) {
        return activities.iter().collect();
    }

    let term = search_term.expect("checked non-blank above");
    activities
        .iter()
        .filter(|a| matches_search(a, term, activity_types, known_locations))
        .collect()
}

/// Removes activities whose type is private, unless the active search term
/// matches that type's name. A blank term removes all private activities.
///
/// An activity whose type cannot be resolved is treated as public and passes
/// through, matching the C#.
pub fn filter_private<'a>(
    activities: &'a [Activity],
    search_term: Option<&str>,
    activity_types: &[ActivityType],
) -> Vec<&'a Activity> {
    let term = if is_blank(search_term) {
        None
    } else {
        search_term
    };

    activities
        .iter()
        .filter(|activity| {
            let activity_type = activity_types
                .iter()
                .find(|t| t.id == activity.activity_type_id);

            match activity_type {
                None => true,
                Some(t) if !t.is_private => true,
                Some(t) => term.is_some_and(|term| contains_ignore_case(&t.name, term)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::TrainerTime;
    use crate::models::NetBenefit;

    fn activity_type(id: i32, name: &str) -> ActivityType {
        ActivityType {
            id,
            name: name.to_owned(),
            net_benefit: NetBenefit::Neutral,
            daily_amount: None,
            weekly_amount: None,
            unit: None,
            is_private: false,
            decimal_places: 0,
        }
    }

    fn private_type(id: i32, name: &str) -> ActivityType {
        ActivityType {
            is_private: true,
            ..activity_type(id, name)
        }
    }

    fn types() -> Vec<ActivityType> {
        vec![
            activity_type(1, "Running"),
            activity_type(2, "Swimming"),
            activity_type(3, "Reading"),
        ]
    }

    fn types_with_private() -> Vec<ActivityType> {
        vec![
            activity_type(1, "Running"),
            private_type(2, "Meditation"),
            activity_type(3, "Swimming"),
        ]
    }

    fn locations() -> Vec<KnownLocation> {
        vec![
            KnownLocation {
                id: 10,
                name: "My Gym".to_owned(),
                latitude: 0.0,
                longitude: 0.0,
            },
            KnownLocation {
                id: 20,
                name: "Home Pool".to_owned(),
                latitude: 0.0,
                longitude: 0.0,
            },
        ]
    }

    fn activity(
        id: i32,
        activity_type_id: i32,
        amount: i32,
        notes: Option<&str>,
        known_location_id: Option<i32>,
    ) -> Activity {
        Activity {
            id,
            activity_type_id,
            when: TrainerTime::parse("2026-01-01T08:00:00Z").expect("parses"),
            amount,
            notes: notes.map(str::to_owned),
            duration_seconds: None,
            known_location_id,
        }
    }

    fn ids(result: &[&Activity]) -> Vec<i32> {
        result.iter().map(|a| a.id).collect()
    }

    // ── Blank search returns input unchanged ──────────────────────────────

    #[test]
    fn blank_search_returns_input_unchanged() {
        let activities = vec![activity(1, 1, 5, Some("note"), None)];
        for term in [None, Some(""), Some(" "), Some("   ")] {
            let result = filter_by_search(&activities, term, &types(), &[]);
            assert_eq!(ids(&result), vec![1], "{term:?}");
        }
    }

    // ── Matching by each field ────────────────────────────────────────────

    #[test]
    fn matches_by_activity_type_name() {
        let activities = vec![activity(1, 1, 5, None, None), activity(2, 2, 5, None, None)];
        let result = filter_by_search(&activities, Some("Running"), &types(), &[]);
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn matches_by_notes() {
        let activities = vec![activity(1, 1, 5, Some("morning jog"), None)];
        let result = filter_by_search(&activities, Some("jog"), &types(), &[]);
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn matches_by_amount() {
        let activities = vec![
            activity(1, 1, 15, Some(""), None),
            activity(2, 2, 20, Some(""), None),
        ];
        let result = filter_by_search(&activities, Some("15"), &types(), &[]);
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn matches_an_amount_substring() {
        let activities = vec![activity(1, 1, 150, Some(""), None)];
        let result = filter_by_search(&activities, Some("15"), &types(), &[]);
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn amount_matching_is_a_substring_test_not_equality() {
        // Consequence of the two scenarios above: a search for "15" matches both
        // 15 and 150, because the displayed amount is compared by substring.
        let activities = vec![
            activity(1, 1, 15, None, None),
            activity(2, 1, 150, None, None),
        ];
        let result = filter_by_search(&activities, Some("15"), &types(), &[]);
        assert_eq!(ids(&result), vec![1, 2]);
    }

    #[test]
    fn type_name_match_is_case_insensitive() {
        let activities = vec![activity(1, 1, 5, None, None)];
        for term in ["RUN", "run", "Running"] {
            let result = filter_by_search(&activities, Some(term), &types(), &[]);
            assert_eq!(ids(&result), vec![1], "{term}");
        }
    }

    #[test]
    fn notes_match_is_case_insensitive() {
        let activities = vec![activity(1, 1, 5, Some("Morning Jog"), None)];
        let result = filter_by_search(&activities, Some("MORNING"), &types(), &[]);
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn no_matches_returns_empty() {
        let activities = vec![activity(1, 1, 5, Some("note"), None)];
        let result = filter_by_search(&activities, Some("zzz"), &types(), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn only_matching_activities_are_returned() {
        let activities = vec![
            activity(1, 1, 5, None, None),
            activity(2, 2, 5, None, None),
            activity(3, 3, 5, None, None),
        ];
        let result = filter_by_search(&activities, Some("read"), &types(), &[]);
        assert_eq!(ids(&result), vec![3]);
    }

    #[test]
    fn matches_across_type_and_notes_together() {
        let activities = vec![
            activity(1, 1, 5, None, None),
            activity(2, 2, 5, Some("running late"), None),
        ];
        let result = filter_by_search(&activities, Some("running"), &types(), &[]);
        assert_eq!(ids(&result), vec![1, 2]);
    }

    // ── Missing or empty reference data ───────────────────────────────────

    #[test]
    fn null_notes_do_not_panic_and_amount_still_matches() {
        let activities = vec![activity(1, 1, 42, None, None)];
        assert_eq!(
            ids(&filter_by_search(&activities, Some("42"), &types(), &[])),
            vec![1]
        );
        assert!(filter_by_search(&activities, Some("zzz"), &types(), &[]).is_empty());
    }

    #[test]
    fn missing_activity_type_treats_the_name_as_empty() {
        let activities = vec![activity(1, 99, 42, Some("note"), None)];
        assert_eq!(
            ids(&filter_by_search(&activities, Some("note"), &types(), &[])),
            vec![1]
        );
        assert_eq!(
            ids(&filter_by_search(&activities, Some("42"), &types(), &[])),
            vec![1]
        );
        assert!(filter_by_search(&activities, Some("Running"), &types(), &[]).is_empty());
    }

    #[test]
    fn empty_activities_list_returns_empty() {
        assert!(filter_by_search(&[], Some("anything"), &types(), &[]).is_empty());
        assert!(filter_by_search(&[], None, &types(), &[]).is_empty());
    }

    #[test]
    fn empty_activity_types_list_matches_only_notes_or_amount() {
        let activities = vec![activity(1, 1, 42, Some("note"), None)];
        assert_eq!(
            ids(&filter_by_search(&activities, Some("note"), &[], &[])),
            vec![1]
        );
        assert_eq!(
            ids(&filter_by_search(&activities, Some("42"), &[], &[])),
            vec![1]
        );
        assert!(filter_by_search(&activities, Some("Running"), &[], &[]).is_empty());
    }

    // ── Location matching ─────────────────────────────────────────────────

    #[test]
    fn matches_by_location_name_case_insensitively() {
        let activities = vec![activity(1, 1, 5, None, Some(10))];
        for term in ["Gym", "gym", "MY GYM"] {
            let result = filter_by_search(&activities, Some(term), &types(), &locations());
            assert_eq!(ids(&result), vec![1], "{term}");
        }
    }

    #[test]
    fn location_name_not_matching_excludes_the_activity() {
        let activities = vec![activity(1, 1, 5, None, Some(10))];
        let result = filter_by_search(&activities, Some("Pool"), &types(), &locations());
        assert!(result.is_empty());
    }

    #[test]
    fn empty_locations_or_absent_id_treat_the_name_as_empty() {
        let with_id = vec![activity(1, 1, 5, None, Some(10))];
        assert!(filter_by_search(&with_id, Some("Gym"), &types(), &[]).is_empty());

        let without_id = vec![activity(1, 1, 5, None, None)];
        assert!(filter_by_search(&without_id, Some("Gym"), &types(), &locations()).is_empty());
    }

    // ── Decimal amounts (ActivitySearchFilterDecimalTests) ────────────────

    fn decimal_types() -> Vec<ActivityType> {
        let mut water = activity_type(1, "Water");
        water.unit = Some("L".to_owned());
        water.decimal_places = 2;
        vec![water]
    }

    fn decimal_activities() -> Vec<Activity> {
        vec![
            activity(1, 1, 125, None, None), // displays as 1.25
            activity(2, 1, 50, None, None),  // displays as 0.5
        ]
    }

    #[test]
    fn search_matches_the_decimal_form() {
        let activities = decimal_activities();
        let result = filter_by_search(&activities, Some("1.25"), &decimal_types(), &[]);
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn search_matches_the_trimmed_decimal_form() {
        // 50 at 2 places displays as "0.5", not "0.50".
        let activities = decimal_activities();
        let result = filter_by_search(&activities, Some("0.5"), &decimal_types(), &[]);
        assert_eq!(ids(&result), vec![2]);
    }

    #[test]
    fn the_raw_stored_integer_does_not_match_a_scaled_amount() {
        let activities = decimal_activities();
        let result = filter_by_search(&activities, Some("125"), &decimal_types(), &[]);
        assert!(result.is_empty());
    }

    // ── FilterPrivate ─────────────────────────────────────────────────────

    #[test]
    fn blank_search_hides_private_activities() {
        let activities = vec![activity(1, 1, 5, None, None), activity(2, 2, 5, None, None)];
        for term in [None, Some(""), Some("   ")] {
            let result = filter_private(&activities, term, &types_with_private());
            assert_eq!(ids(&result), vec![1], "{term:?}");
        }
    }

    #[test]
    fn search_matching_a_private_type_name_reveals_it() {
        let activities = vec![activity(1, 1, 5, None, None), activity(2, 2, 5, None, None)];
        let result = filter_private(&activities, Some("Meditation"), &types_with_private());
        assert_eq!(ids(&result), vec![1, 2]);
    }

    #[test]
    fn search_not_matching_a_private_type_name_still_hides_it() {
        let activities = vec![activity(1, 1, 5, None, None), activity(2, 2, 5, None, None)];
        let result = filter_private(&activities, Some("Running"), &types_with_private());
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn private_type_name_match_is_case_insensitive() {
        let activities = vec![activity(2, 2, 5, None, None)];
        let result = filter_private(&activities, Some("MEDITATION"), &types_with_private());
        assert_eq!(ids(&result), vec![2]);
    }

    #[test]
    fn public_activities_always_pass_through() {
        let activities = vec![activity(1, 1, 5, None, None), activity(3, 3, 5, None, None)];
        let result = filter_private(&activities, Some("zzz"), &types_with_private());
        assert_eq!(ids(&result), vec![1, 3]);
    }

    #[test]
    fn activities_with_an_unresolvable_type_pass_through() {
        let activities = vec![activity(1, 99, 5, None, None)];
        let result = filter_private(&activities, None, &types_with_private());
        assert_eq!(ids(&result), vec![1]);
    }

    #[test]
    fn home_chart_call_pattern_excludes_private_types() {
        let activities = vec![
            activity(1, 1, 5, None, None),
            activity(2, 2, 5, None, None),
            activity(3, 3, 5, None, None),
        ];
        let result = filter_private(&activities, None, &types_with_private());
        assert_eq!(ids(&result), vec![1, 3]);
        assert!(!result.iter().any(|a| a.activity_type_id == 2));
    }
}
