//! Lazy week-by-week loading, ported from `Trainer/Services/WeekFillLoader.cs`.
//!
//! Exists for the "all time plus an active search" case (issue #85). The search
//! filter runs client-side over already-loaded weeks, so matches in older weeks
//! never surface unless loading continues — and a sparse result set leaves the
//! page unscrollable, so the intersection observer never re-fires either.

use std::collections::BTreeSet;
use std::future::Future;

/// The most recent available week not yet loaded, or `None` when all are.
///
/// Week keys are zero-padded `YYYY.WW`, so descending lexical order is
/// reverse-chronological.
pub fn next_week_key(available: &[String], loaded: &BTreeSet<String>) -> Option<String> {
    available
        .iter()
        .filter(|key| !loaded.contains(*key))
        .max()
        .cloned()
}

/// Loads successive most-recent unloaded weeks until `displayed_count` reports
/// at least `min_displayed`, or no unloaded week remains.
///
/// `load_week` is responsible both for loading the week and for recording its
/// key in `loaded`; that is what guarantees the loop advances and terminates.
///
/// Returns whether unloaded weeks still remain — that is, whether there is more
/// to load.
pub async fn fill<F, Fut, C>(
    available: &[String],
    loaded: &mut BTreeSet<String>,
    mut load_week: F,
    mut displayed_count: C,
    min_displayed: usize,
) -> bool
where
    F: FnMut(String, &mut BTreeSet<String>) -> Fut,
    Fut: Future<Output = ()>,
    C: FnMut() -> usize,
{
    while displayed_count() < min_displayed {
        let Some(next) = next_week_key(available, loaded) else {
            break;
        };
        load_week(next, loaded).await;
    }

    available.iter().any(|key| !loaded.contains(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::block_on;
    use std::cell::Cell;

    fn weeks(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|k| (*k).to_owned()).collect()
    }

    /// Ports `NextWeekKey_ReturnsMostRecentUnloadedWeek`.
    #[test]
    fn picks_the_most_recent_unloaded_week() {
        let available = weeks(&["2026.01", "2026.05", "2026.03"]);
        let loaded = BTreeSet::from(["2026.05".to_owned()]);
        assert_eq!(
            next_week_key(&available, &loaded),
            Some("2026.03".to_owned())
        );
    }

    /// Ports `NextWeekKey_ReturnsNull_WhenAllWeeksLoaded`.
    #[test]
    fn returns_none_once_every_week_is_loaded() {
        let available = weeks(&["2026.01", "2026.02"]);
        let loaded: BTreeSet<String> = available.iter().cloned().collect();
        assert_eq!(next_week_key(&available, &loaded), None);
    }

    #[test]
    fn ordering_is_chronological_across_a_year_boundary() {
        let available = weeks(&["2025.53", "2026.01"]);
        let loaded = BTreeSet::new();
        assert_eq!(
            next_week_key(&available, &loaded),
            Some("2026.01".to_owned())
        );
    }

    /// Ports `FillAsync_LoadsOlderWeeks_UntilMatchesSurface`.
    #[test]
    fn loads_older_weeks_until_enough_matches_surface() {
        block_on(async {
            let available = weeks(&["2026.01", "2026.02", "2026.03"]);
            let mut loaded = BTreeSet::new();
            let matches = Cell::new(0usize);

            // Only the oldest week holds a match.
            let more = fill(
                &available,
                &mut loaded,
                |key, loaded| {
                    let found = key == "2026.01";
                    loaded.insert(key);
                    if found {
                        matches.set(matches.get() + 1);
                    }
                    async {}
                },
                || matches.get(),
                1,
            )
            .await;

            assert_eq!(matches.get(), 1);
            assert_eq!(loaded.len(), 3, "kept loading back to the oldest week");
            assert!(!more, "nothing remains unloaded");
        });
    }

    /// Ports `FillAsync_LoadsAllWeeks_WhenNoMatchesExistAnywhere`.
    #[test]
    fn stops_after_exhausting_every_week_when_nothing_matches() {
        block_on(async {
            let available = weeks(&["2026.01", "2026.02"]);
            let mut loaded = BTreeSet::new();

            let more = fill(
                &available,
                &mut loaded,
                |key, loaded| {
                    loaded.insert(key);
                    async {}
                },
                || 0,
                5,
            )
            .await;

            assert_eq!(loaded.len(), 2);
            assert!(!more);
        });
    }

    /// Ports `FillAsync_DoesNotLoad_WhenThresholdAlreadyMet`.
    #[test]
    fn loads_nothing_when_the_threshold_is_already_met() {
        block_on(async {
            let available = weeks(&["2026.01", "2026.02"]);
            let mut loaded = BTreeSet::new();

            let more = fill(
                &available,
                &mut loaded,
                |key, loaded| {
                    loaded.insert(key);
                    async {}
                },
                || 10,
                5,
            )
            .await;

            assert!(loaded.is_empty(), "no week should have been loaded");
            assert!(more, "unloaded weeks still remain");
        });
    }
}
