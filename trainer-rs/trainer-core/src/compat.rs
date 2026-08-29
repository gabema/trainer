//! Cross-implementation compatibility, section 9.
//!
//! Everything else in this crate verifies one direction: that Rust reproduces
//! what C# wrote. These tests close the loop in both directions, against a
//! fixture whose bytes came start-to-finish from the C# export path rather than
//! through the Python de-identifier.

#[cfg(test)]
mod tests {
    use crate::fixtures::{block_on, read_fixture};
    use crate::models::{ExportDocument, Format, KnownLocation, NetBenefit, to_json};
    use crate::services::export_import::ExportImportService;
    use crate::storage::buckets::WeekBucketed;
    use crate::storage::{MemStorage, Storage};

    fn store() -> WeekBucketed<MemStorage> {
        WeekBucketed::new(MemStorage::new())
    }

    /// Task 9.1 — a C#-produced export must import and re-export unchanged.
    #[test]
    fn a_csharp_produced_export_round_trips_byte_identically() {
        block_on(async {
            let original = read_fixture("csharp-export.json");
            let storage = store();
            let service = ExportImportService::new(&storage);

            service.import(&original).await.expect("imports");

            let document: ExportDocument = serde_json::from_str(&original).expect("fixture parses");
            let exported = service
                .export(document.export_date.naive())
                .await
                .expect("exports");

            assert_eq!(exported, original);
        });
    }

    /// The fixture is only worth round-tripping if it actually covers the
    /// cases that have bitten during this port.
    #[test]
    fn the_csharp_fixture_exercises_the_hard_cases() {
        let raw = read_fixture("csharp-export.json");
        let document: ExportDocument = serde_json::from_str(&raw).expect("parses");

        // Year-boundary bucketing, where calendar-year week keys diverge from ISO.
        assert!(document.activities.contains_key("2025.53"));
        assert!(document.activities.contains_key("2026.01"));

        let activities: Vec<_> = document.activities.values().flatten().collect();
        // All three notes states.
        assert!(activities.iter().any(|a| a.notes.is_none()));
        assert!(activities.iter().any(|a| a.notes.as_deref() == Some("")));
        assert!(
            activities
                .iter()
                .any(|a| a.notes.as_deref().is_some_and(|n| !n.is_empty()))
        );

        // Both timestamp branches.
        assert!(activities.iter().any(|a| a.when.to_wire().ends_with('Z')));
        assert!(activities.iter().any(|a| a.when.to_wire().ends_with("-08")));

        // Escaping: the raw bytes must carry \u escapes, not literal characters.
        assert!(raw.contains(r"\u00E9"), "an accented character");
        assert!(raw.contains(r"\u0026"), "an ampersand");
        assert!(raw.contains(r"\u0027"), "an apostrophe");
        assert!(raw.contains(r"\u00BD"), "a vulgar fraction");
        assert!(!raw.contains("café"), "non-ASCII must not appear literally");

        // A private type with fractional amounts.
        assert!(document.activity_types.iter().any(|t| t.is_private));
        assert!(
            document
                .activity_types
                .iter()
                .any(|t| t.decimal_places == 2)
        );
        assert!(
            document
                .activity_types
                .iter()
                .any(|t| t.net_benefit == NetBenefit::Positive)
        );
    }

    /// Task 9.2 — the Rust-produced export committed for the C# side to import.
    ///
    /// Regenerate with:
    /// ```text
    /// TRAINER_GENERATE_FIXTURES=1 cargo test -p trainer-core rust_produced_export
    /// ```
    /// The C# test `ImportDataAsync_AcceptsRustProducedExport` reads the result,
    /// so the two implementations meet on a real file rather than by assertion.
    #[test]
    fn rust_produced_export_is_written_for_the_csharp_side() {
        block_on(async {
            let storage = store();
            let service = ExportImportService::new(&storage);

            // Built from the C# fixture so the content is known-good, then
            // re-emitted by the Rust serializer.
            let source = read_fixture("csharp-export.json");
            service.import(&source).await.expect("imports");

            // An extra location with a whole-valued coordinate, the case where
            // .NET drops the fractional part and serde_json would not.
            crate::services::known_location::KnownLocationService::new(storage.inner())
                .save(KnownLocation {
                    id: 12345,
                    name: "Whole \"quoted\" <spot>".to_owned(),
                    latitude: 10.0,
                    longitude: -20.0,
                })
                .await
                .expect("saves");

            let document: ExportDocument = serde_json::from_str(&source).expect("parses");
            let exported = service
                .export(document.export_date.naive())
                .await
                .expect("exports");

            if std::env::var("TRAINER_GENERATE_FIXTURES").is_ok() {
                std::fs::write(
                    crate::fixtures::fixture_dir().join("rust-export.json"),
                    &exported,
                )
                .expect("writes the fixture");
            }

            // The committed file must match what this run produces, so it cannot
            // silently drift away from the implementation it represents.
            let committed = read_fixture("rust-export.json");
            assert_eq!(
                exported, committed,
                "rust-export.json is stale; regenerate with TRAINER_GENERATE_FIXTURES=1"
            );

            // Asserted here rather than in a separate test, which would depend
            // on this one having already written the file.
            assert!(
                exported.contains(r#""latitude":10,"#),
                "a whole double must be written as 10, not 10.0"
            );
            assert!(
                exported.contains(r"\u0022"),
                "a quote uses the unicode escape"
            );
            assert!(exported.contains(r"\u003C"), "a less-than is escaped");
        });
    }

    /// `active-activities`: "Active activity state is excluded from import/export".
    #[test]
    fn active_activity_state_never_enters_an_export() {
        block_on(async {
            let storage = store();
            ExportImportService::new(&storage)
                .import(&read_fixture("csharp-export.json"))
                .await
                .expect("imports");

            // Start a timer, which persists to its own localStorage key.
            let side_store = MemStorage::new();
            let active = crate::services::active_activity::ActiveActivityService::new(&side_store);
            active
                .start(
                    1,
                    crate::services::active_time::ActiveTime::parse("2026-08-28T15:43:21-07:00")
                        .expect("valid"),
                )
                .await
                .expect("starts");
            assert!(side_store.contains("trainer_active_activities"));

            let exported = ExportImportService::new(&storage)
                .export(
                    chrono::NaiveDate::from_ymd_opt(2026, 8, 28)
                        .expect("valid")
                        .and_hms_opt(0, 0, 0)
                        .expect("valid"),
                )
                .await
                .expect("exports");

            assert!(
                !exported.contains("trainer_active_activities"),
                "the export must not carry active-timer state"
            );
            assert!(!exported.contains("startTime"));
        });
    }

    /// Task 9.3's native half: data written through the storage layer is
    /// readable by a fresh service instance, and every model survives.
    #[test]
    fn data_written_by_rust_reads_back_through_a_fresh_service() {
        block_on(async {
            let storage = store();
            ExportImportService::new(&storage)
                .import(&read_fixture("csharp-export.json"))
                .await
                .expect("imports");

            let raw = storage.inner().snapshot();
            let reloaded = WeekBucketed::new(MemStorage::seeded(raw));

            let aggregate = reloaded
                .get_item("activities")
                .await
                .expect("ok")
                .expect("present");
            let activities: Vec<crate::models::Activity> =
                serde_json::from_str(&aggregate).expect("parses");
            assert_eq!(activities.len(), 4);

            let types = crate::services::activity_type::ActivityTypeService::new(reloaded.inner())
                .all()
                .await
                .expect("ok");
            assert_eq!(types.len(), 2);
        });
    }

    /// Storage-format output must also be re-readable, not only export format.
    #[test]
    fn both_serializer_configurations_are_mutually_readable() {
        let raw = read_fixture("csharp-export.json");
        let document: ExportDocument = serde_json::from_str(&raw).expect("parses");
        let activities: Vec<_> = document.activities.values().flatten().cloned().collect();

        let storage_form = to_json(&activities, Format::Storage).expect("serializes");
        let export_form = to_json(&activities, Format::Export).expect("serializes");
        assert_ne!(storage_form, export_form, "the two forms must differ");

        // Each parses back to the same values despite the different bytes.
        let from_storage: Vec<crate::models::Activity> =
            serde_json::from_str(&storage_form).expect("parses");
        let from_export: Vec<crate::models::Activity> =
            serde_json::from_str(&export_form).expect("parses");
        assert_eq!(from_storage, from_export);
        assert_eq!(from_storage, activities);
    }
}
