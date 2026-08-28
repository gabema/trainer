## 1. Golden fixtures (before any implementation)

- [ ] 1.1 Add a temporary C# console/test harness that dumps `(date, weekKey)` pairs from `WeekHelper.GetWeekKey` for every day across a 20-year span covering multiple year boundaries; commit the output as `trainer-rs/tests/fixtures/week-keys.csv`
- [ ] 1.2 Curate a data export from a running Blazor instance covering: activities near year boundaries, fractional amounts, private activity types, durations, known locations, empty notes, null notes, and an in-progress activity; commit as `trainer-rs/tests/fixtures/export.json`
- [ ] 1.3 Capture a second export from a profile in a non-UTC timezone, including at least one timestamp with a non-zero-minute UTC offset to exercise the `+05:30` formatting path; commit as `trainer-rs/tests/fixtures/export-offset.json`
- [ ] 1.4 Dump the raw IndexedDB contents of a real profile (keys plus values) as `trainer-rs/tests/fixtures/idb-snapshot.json` to pin the on-disk representation independently of the export format
- [ ] 1.5 Remove the temporary C# dump harness once fixtures are committed

## 2. Crate scaffolding

- [ ] 2.1 Create `trainer-rs/` crate targeting `wasm32-unknown-unknown` with `serde`, `serde_json`, `chrono`, `async-trait`
- [ ] 2.2 Add browser-facing dependencies gated to `wasm32`: `wasm-bindgen`, `js-sys`, `web-sys` (IndexedDB features only), `wasm-bindgen-futures`, `wasm-bindgen-test`
- [ ] 2.3 Verify `cargo test` runs natively and `wasm-bindgen-test` runs in headless Chrome with a placeholder test in each tier
- [ ] 2.4 Add `trainer-rs/target` and build artifacts to `.gitignore`

## 3. Timestamp serialization

- [ ] 3.1 Implement the serde serializer reproducing `DateTimeConverter.Write` and `FormatOffset`, including hour-only offsets when the minute component is zero and `Z` for zero offset
- [ ] 3.2 Implement the deserializer reproducing the hour-only-offset regex normalization and the zero-offset-means-UTC read behavior
- [ ] 3.3 Decide and document the Rust representation for .NET's `DateTimeKind` distinction (likely instant plus offset); record the choice in `design.md` under Decisions
- [ ] 3.4 Assert both export fixtures round-trip byte-identically through serialize/deserialize

## 4. Domain models

- [ ] 4.1 Port `Activity`, `ActivityType`, `KnownLocation`, `NetBenefit`, `DurationOption` with camelCase serde naming and non-indented output
- [ ] 4.2 Port `EmptyStringAsNullConverter` behavior for fields where the C# implementation applied it
- [ ] 4.3 Assert serialized output of each model matches the corresponding fragment of the export fixture byte-for-byte

## 5. Week keys

- [ ] 5.1 Implement .NET's `FirstFourDayWeek` / Monday week-of-year rule paired with the plain calendar year — explicitly NOT `chrono::IsoWeek`
- [ ] 5.2 Assert every pair in `week-keys.csv` reproduces exactly
- [ ] 5.3 Port `GetWeekStartDate` including the linear scan from January 1st and its silent fallback when no date matches
- [ ] 5.4 Port `GetWeekEndDate`, `GetWeekKeysInRange`, `GetStorageKey`, `ExtractWeekKey`
- [ ] 5.5 Port `WeekHelperTests` scenario-for-scenario

## 6. Pure helpers

- [ ] 6.1 Port `DateTimeHelper` and its formatting behavior
- [ ] 6.2 Port `DecimalAmount` and `DecimalPlacesWarning`, covering the `fractional-activity-amounts` spec scenarios
- [ ] 6.3 Port `DurationInput` parsing, covering the `activity-duration` spec scenarios including `0:30`, plain minutes, and out-of-range seconds
- [ ] 6.4 Port `ActivityAmountDisplay.FormatDuration`, including the unpadded single-digit seconds requirement
- [ ] 6.5 Port `ActivitySearchFilter`, covering the `activity-filtering` spec scenarios
- [ ] 6.6 Port `StringExtensions`
- [ ] 6.7 Port the corresponding helper tests scenario-for-scenario

## 7. Storage seam

- [ ] 7.1 Define the `Storage` trait as `#[async_trait(?Send)]` mirroring `IStorageService`
- [ ] 7.2 Implement portable `MemStorage` to stand in for Moq in service tests
- [ ] 7.3 Implement the `IdbRequest → Future` adapter with `Closure` plus oneshot, confined to one module
- [ ] 7.4 Implement `IdbStorage` against database `Trainer` v1, object store `activities`, preserving the `js_sys::JSON` parse-on-write / stringify-on-read boundary
- [ ] 7.5 Implement week-bucketed read/write, including removal of emptied buckets
- [ ] 7.6 Implement the one-time localStorage-to-IndexedDB migration, non-fatal on failure
- [ ] 7.7 Verify `IdbStorage` against `idb-snapshot.json` under `wasm-bindgen-test`: values read back as objects, database is not upgraded, keys match
- [ ] 7.8 Port `LocalStorageServiceTests` scenario-for-scenario

## 8. Services

- [ ] 8.1 Port `ActivityService` including `RecalculateNextIdAsync` and week-range queries
- [ ] 8.2 Port `ActivityTypeService`
- [ ] 8.3 Port `GoalService`, covering the `neutral-benefit` spec scenarios
- [ ] 8.4 Port `KnownLocationService` including `FindNearbyAsync` and `NextAutoNameAsync`, covering the `known-locations` spec scenarios
- [ ] 8.5 Port `ActiveActivityService` state and persistence to `trainer_active_activities`, preserving the `id` / `startTime` entry shape and silent recovery from corrupt state; model the change/tick notifications as signals rather than events
- [ ] 8.6 Port `ExportImportService`, preserving the export file format
- [ ] 8.7 Port `WeekFillLoader`
- [ ] 8.8 Port `ActivityServiceTests`, `KnownLocationServiceTests`, and `ActiveActivityServiceTests` scenario-for-scenario against `MemStorage`

## 9. Cross-implementation compatibility

- [ ] 9.1 Assert a C#-produced export imports into the Rust implementation with identical field values
- [ ] 9.2 Assert a Rust-produced export imports into the C# implementation with identical field values
- [ ] 9.3 Assert data written by Rust is readable by the existing JavaScript shim, and vice versa
- [ ] 9.4 Walk all nine existing capability specs and confirm every scenario has a corresponding ported test or is explicitly deferred to `rust-ui` as view-layer behavior

## 10. CI

- [ ] 10.1 Add Rust toolchain, `wasm32-unknown-unknown` target, and `Swatinem/rust-cache` to `test.yml`
- [ ] 10.2 Add `cargo test` for the native tier
- [ ] 10.3 Add headless Chrome setup and the `wasm-bindgen-test` tier, with `wasm-bindgen-cli` pinned to the `wasm-bindgen` crate version
- [ ] 10.4 Add `cargo fmt --check` and `cargo clippy -- -D warnings`, matching the strictness of the existing `CodeAnalysisTreatWarningsAsErrors` setting
- [ ] 10.5 Confirm the existing .NET build and test steps still run and pass alongside the Rust steps
- [ ] 10.6 Confirm `deploy.yml` is unmodified and the Blazor app still publishes
