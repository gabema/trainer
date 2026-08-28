//! Domain, helpers, services, and the storage seam for the Trainer app.
//!
//! This crate is deliberately free of browser dependencies so its tests run
//! natively under `cargo test`. Browser-facing code lives in `trainer-web`.

/// Identifies this crate to the browser tier, which asserts the two are linked.
pub const CRATE_NAME: &str = "trainer-core";

#[cfg(feature = "test-support")]
pub mod fixtures;

#[cfg(test)]
mod tier_check {
    //! Verifies the native test tier is wired up and that the committed golden
    //! fixtures are reachable. Sections 3-8 assert against these files, so a
    //! broken path here would surface as confusing failures much later.

    use std::path::PathBuf;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("trainer-core has a parent directory")
            .join("tests")
            .join("fixtures")
    }

    #[test]
    fn week_key_fixture_is_readable() {
        let csv = std::fs::read_to_string(fixture_dir().join("week-keys.csv"))
            .expect("week-keys.csv is committed and readable");
        let mut lines = csv.lines();

        assert_eq!(lines.next(), Some("date,weekKey"));

        let rows: Vec<&str> = lines.collect();
        assert_eq!(
            rows.len(),
            11323,
            "expected every day from 2010-01-01 to 2040-12-31"
        );

        // The year-boundary quirk this port must reproduce: January 1st 2010
        // lands in week 53 of calendar year 2010, where true ISO 8601 numbering
        // would call it 2009-W53.
        assert_eq!(rows[0], "2010-01-01,2010.53");
    }

    #[test]
    fn export_fixture_parses_as_json() {
        let json = std::fs::read_to_string(fixture_dir().join("export.json"))
            .expect("export.json is committed and readable");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("export.json is valid JSON");

        let buckets = value["activities"]
            .as_object()
            .expect("activities is an object keyed by week");
        assert_eq!(buckets.len(), 31);

        // The de-identified fixture keeps the real bucket layout, including the
        // anomalous first-week-of-year bucket.
        assert!(buckets.contains_key("2026.01"));
    }

    #[test]
    fn idb_snapshot_records_value_representation() {
        let json = std::fs::read_to_string(fixture_dir().join("idb-snapshot.json"))
            .expect("idb-snapshot.json is committed and readable");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("idb-snapshot.json is valid JSON");

        // Storage holds structured-cloned values, not strings. The whole storage
        // design depends on this, so it is asserted rather than assumed.
        assert_eq!(
            value["entries"]["activities-2026.01"]["typeofValue"],
            "object"
        );
        assert_eq!(
            value["entries"]["activities-2026.01"]["constructor"],
            "Array"
        );

        // ...except activityNextId, which is stored as a bare number.
        assert_eq!(value["entries"]["activityNextId"]["typeofValue"], "number");
    }
}
