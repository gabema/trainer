## 1. Golden fixtures (before any implementation)

- [x] 1.1 Add a temporary C# console/test harness that dumps `(date, weekKey)` pairs from `WeekHelper.GetWeekKey` for every day across a 20-year span covering multiple year boundaries; commit the output as `trainer-rs/tests/fixtures/week-keys.csv`
- [x] 1.1a Dump `GetWeekStartDate` / `GetWeekEndDate` golden values as `week-boundaries.csv` — needed by tasks 5.3 and 5.4, which run after the harness is deleted at 1.5
- [x] 1.1b Dump the year-boundary round-trip failures as `week-key-anomalies.csv` so the defect is pinned rather than rediscovered during the port
- [x] 1.2 De-identify a real export into `trainer-rs/tests/fixtures/export.json` via `deidentify.py`, preserving every structural property (527 activities, 8 field-presence combinations, both hour-only offsets, netBenefit 0/1/2, decimalPlaces 0/2, both `isPrivate` values, signed knownLocation ids, the `2026.01` boundary bucket) while replacing names, units, note text, and coordinates. Real personal data is not committed
- [x] 1.3 Generate timestamp fixtures by driving the real `DateTimeConverter` under different `TZ` values, covering all three offset-formatting branches: hour-only (`-08`/`-07`), non-zero-minute (`+05:30` Kolkata, `+08:45` Eucla), and zero (`Z`)
- [x] 1.3a Capture both serializer configurations separately — `timestamps-export-*` (nulls omitted) and `timestamps-storage-*` (nulls written) — because `ExportImportService` sets `DefaultIgnoreCondition` and `IndexedDbStorageService` does not
- [x] 1.3b Capture `timestamps-roundtrip-*` recording what each emitted string parses back to through the converter's own `Read` path
- [x] 1.4 De-identify a raw IndexedDB dump into `trainer-rs/tests/fixtures/idb-snapshot.json`. Confirms values are structured-cloned Arrays (33 entries), that `activityNextId` is a bare JS Number, and that storage writes every optional field explicitly as `null`
- [ ] 1.4a Synthesize fixtures for the legacy localStorage migration and for `trainer_active_activities`. The captured profile has an **empty** localStorage, so neither the migration path (task 7.6) nor the active-activity persistence format (task 8.5) can be validated against real data
- [ ] 1.5 Remove the temporary C# dump harness — **deferred until section 8 is complete**. Sections 3–8 may still need golden values, and the harness cannot be recreated after deletion without another capture. Originally sequenced here, which would have stranded tasks 5.3 and 5.4

## 2. Crate scaffolding

- [x] 2.1 Create the `trainer-rs/` workspace with `serde`, `serde_json`, `chrono`, `async-trait`, split into `trainer-core` (pure, native tests) and `trainer-web` (browser). The split makes the native/browser boundary a compile error rather than a review discipline
- [x] 2.2 Add browser-facing dependencies to `trainer-web`, gated to `wasm32`: `wasm-bindgen` 0.2.127, `js-sys`, `web-sys` (IndexedDB features only), `wasm-bindgen-futures`, `wasm-bindgen-test`
- [x] 2.3a Native tier verified: `cargo test -p trainer-core` green, with tests asserting the committed fixtures are readable and record the expected week-key, bucket, and value-representation facts
- [x] 2.3b Browser tier compiles to wasm32 and `.cargo/config.toml` routes it through `wasm-bindgen-test-runner`, pinned to matching CLI 0.2.127
- [x] 2.3c Browser tier executes in headless Chrome — ChromeDriver 152.0.7977.64 against Chrome 152.0.7977.64, 2 tests green. Homebrew's driver needed its macOS Gatekeeper quarantine attribute cleared before it would launch (it was SIGKILLed, exit 137); this affects local runs only, since GitHub's Ubuntu runners have no Gatekeeper
- [x] 2.4 Add `trainer-rs/target/` and `**/*.rs.bk` to `.gitignore`. `Cargo.lock` is deliberately committed — this builds a deployed application, not a library

## 3. Timestamp serialization

- [ ] 3.1 Implement the serde serializer reproducing `DateTimeConverter.Write` and `FormatOffset`, including hour-only offsets when the minute component is zero and `Z` for zero offset
- [ ] 3.2 Implement the deserializer reproducing the hour-only-offset regex normalization and the zero-offset-means-UTC read behavior
- [ ] 3.3 Decide and document the Rust representation for .NET's `DateTimeKind` distinction (likely instant plus offset); record the choice in `design.md` under Decisions
- [ ] 3.4 Assert `export.json` and every `timestamps-export-*` / `timestamps-storage-*` fixture round-trips byte-identically through serialize/deserialize under its corresponding configuration

## 4. Domain models

- [ ] 4.1 Port `Activity`, `ActivityType`, `KnownLocation`, `NetBenefit`, `DurationOption` with camelCase serde naming, non-indented output, and field order matching the C# record declaration order
- [ ] 4.1a Model `notes` so all three observed states stay distinct: `None`, `Some("")`, and `Some(text)`. In the real profile these occur 50 / 38 / 439 times; collapsing empty string to `None` would corrupt 38 activities
- [ ] 4.2 Implement TWO serde configurations: export (skip `None`) and storage (serialize `None` as `null`). Do NOT port `EmptyStringAsNullConverter` — it is dead code, registered in no `JsonSerializerOptions` and applied via no attribute, which is why empty notes serialize as `""` rather than being nulled and omitted
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
- [ ] 7.4 Implement `IdbStorage` against database `Trainer` v1, object store `activities`, preserving the `js_sys::JSON` parse-on-write / stringify-on-read boundary. Open **without** an explicit version so no upgrade transaction can fire. Handle scalar values: `activityNextId` is stored as a bare JS Number, not an object or array
- [ ] 7.4a Cover all four storage key shapes seen in the real profile: `activities-{weekKey}` (Array), `activityTypes` (Array), `knownLocations` (Array), `activityNextId` (Number)
- [ ] 7.5 Implement week-bucketed read/write, including removal of emptied buckets
- [ ] 7.6 Implement the one-time localStorage-to-IndexedDB migration, non-fatal on failure
- [ ] 7.7 Verify `IdbStorage` against `idb-snapshot.json` under `wasm-bindgen-test`: values read back as objects, database is not upgraded, keys match
- [ ] 7.8 Port `LocalStorageServiceTests` scenario-for-scenario

## 8. Services

- [ ] 8.1 Port `ActivityService` including `RecalculateNextIdAsync` and week-range queries
- [ ] 8.2 Port `ActivityTypeService`
- [ ] 8.3 Port `GoalService`, covering the `neutral-benefit` spec scenarios
- [ ] 8.4 Port `KnownLocationService` including `FindNearbyAsync` (Haversine, 100m threshold) and `NextAutoNameAsync`. Do NOT attempt to reproduce `AssignId` — it uses `HashCode.Combine`, which .NET seeds randomly per process, so its output is not reproducible by any implementation including itself. Preserve stored ids verbatim; generate new ones by any collision-avoiding scheme
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
- [ ] 10.4 Add `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` for BOTH the host and `wasm32-unknown-unknown` targets — the wasm-gated test module is invisible to a host-only clippy run. Matches the strictness of the existing `CodeAnalysisTreatWarningsAsErrors` setting; both are clean as of section 2
- [ ] 10.5 Confirm the existing .NET build and test steps still run and pass alongside the Rust steps
- [ ] 10.6 Confirm `deploy.yml` is unmodified and the Blazor app still publishes
