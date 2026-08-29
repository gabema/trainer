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

- [x] 3.1 Implement the serde serializer reproducing `DateTimeConverter.Write` and `FormatOffset`, including hour-only offsets when the minute component is zero and `Z` for zero offset
- [x] 3.2 Implement the deserializer reproducing the hour-only-offset regex normalization (hand-rolled, no `regex` dependency) and the zero-offset-means-UTC read behavior
- [x] 3.3 Represent timestamps as `TrainerTime::Utc(naive)` | `TrainerTime::Offset { naive, offset }`, retaining the parsed offset where the C# reader discards it. Deliberate, documented divergence: byte-identical for all existing data, keeps serialization pure so no ambient timezone is needed, and avoids dragging `js-sys` into `trainer-core`
- [x] 3.4 Round-trip 727 timestamps byte-identically — 200 across 20 timestamp fixtures plus all 527 in `export.json` — with coverage assertions proving all eight offset forms were exercised (`Z`, `-08`, `-07`, `+05:30`, `+05:45`, `+08:45`, `-03:30`, `-02:30`)

- [x] 3.5 Generate cross-zone fixtures pinning the offset-discard behavior, and fixtures for `America/St_Johns` and `Asia/Kathmandu` so negative-offset-with-minutes (`-03:30`, `-02:30`) and `+05:45` exercise the sign/abs interaction in `FormatOffset`

## 4. Domain models

- [x] 4.1 Port `Activity`, `ActivityType`, `KnownLocation`, `NetBenefit`, `DurationOption` with camelCase serde naming, non-indented output, and field order matching the C# record declaration order. `NetBenefit` keeps an `Other(i32)` variant because `System.Text.Json` accepts undefined enum integers
- [x] 4.1a Model `notes` so all three observed states stay distinct: `None`, `Some("")`, and `Some(text)`, asserted at 50 / 38 / 439 against the real profile
- [x] 4.2 Implement TWO serde configurations via the `Fmt` wrapper, since a serde derive cannot switch at runtime: export skips `None`, storage writes `null`. `EmptyStringAsNullConverter` is NOT ported — it is dead code, which is why empty notes serialize as `""`
- [x] 4.3 Assert byte-identity against genuine C# output: the full `export.json` document, plus every `timestamps-export-*` fixture in Export format and every `timestamps-storage-*` fixture in Storage format, with a guard test proving the two formats still differ

- [x] 4.4 Reproduce `System.Text.Json`'s string escaping via a `serde_json` `Formatter`. `JavaScriptEncoder.Default` escapes `"`, `&`, `'`, `+`, `<`, `>`, backtick and everything at U+007F or above; `serde_json` escapes almost none of it. Not a corner case: every positive UTC offset contains `+`, and the real export holds 15 such escapes
- [x] 4.4a Record the authoritative escape table from C# across all 128 ASCII code points plus a non-ASCII spread as `json-escaping.json`, and assert the Rust escaper against every entry
- [x] 4.4b Fix `deidentify.py`, which had destroyed all escape evidence by substituting plain-ASCII notes. It now emits .NET-escaped JSON and carries escape-triggering characters into replacement text, so `export.json` exercises the escaper

## 5. Week keys

- [x] 5.1 Implement .NET's `GetWeekOfYearFullDays` for `FirstFourDayWeek` / Monday, paired with the plain calendar year — explicitly NOT `chrono::IsoWeek`, including the recursion onto 31 December that makes an early-January date report as week 52 or 53
- [x] 5.2 Assert all 11,323 pairs in `week-keys.csv` reproduce exactly
- [x] 5.3 Port `GetWeekStartDate` including the linear scan and the fallback to the 1 January week when no date matches, verified against all 1,636 rows of `week-boundaries.csv` plus the new `week-unmatched-keys.csv`
- [x] 5.4 Port `GetWeekEndDate`, `GetWeekKeysInRange`, `GetStorageKey`, `ExtractWeekKey`, returning `Result` where the C# throws
- [x] 5.5 No `WeekHelperTests` exists to port — `WeekHelper` has **zero** direct unit tests in C#, despite determining every storage key in the app. Its only test-project references are indirect, through `ActivityServiceTests` and `ExportImportServiceTests`. The golden fixtures are its first real coverage, at 11,323 cases
- [x] 5.6 Pin `GetWeekStartDate`'s unmatched-key fallback from C# as `week-unmatched-keys.csv`; no existing fixture reached that branch
- [x] 5.7 Characterise the year-boundary anomaly precisely: it occurs in every year whose 1 January is not a Monday, 27 of 31 years from 2010 to 2040, exceptions 2018/2024/2029/2035. An earlier draft claimed 26 occurrences, one per year without exception

## 6. Pure helpers

- [x] 6.1 Port `DateTimeHelper` as `helpers::when` — `format_elapsed`, `format_when`, `date_range`. The C# formats with `InvariantCulture`, so `chrono`'s English month and meridiem names match directly
- [x] 6.2 Port `DecimalAmount` and `DecimalPlacesWarning` as `helpers::amount`, covering every helper-level `fractional-activity-amounts` scenario
- [x] 6.3 Port `DurationInput` as `helpers::duration`, covering all 10 `activity-duration` scenarios. Parts are trimmed individually because .NET's `int.TryParse` tolerates surrounding whitespace, so `"5 : 30"` parses there and must here
- [x] 6.4 Port `ActivityAmountDisplay` as `helpers::display`, including unpadded single-digit seconds (`5m 5s`, never `5m 05s`)
- [x] 6.5 Port `ActivitySearchFilter` as `helpers::search`, covering every helper-level `activity-filtering` scenario, including matching the *displayed* decimal amount rather than the stored integer
- [x] 6.6 Port `StringExtensions` as `helpers::strings`
- [x] 6.7 Port all seven helper test files scenario-for-scenario: `DateTimeHelperTests`, `DecimalAmountTests`, `DecimalPlacesWarningTests`, `DurationInputTests`, `ActivityAmountDisplayTests`, `ActivitySearchFilterTests`, `ActivitySearchFilterDecimalTests`
- [x] 6.8 Record spec coverage. Fully covered here: all 10 `activity-duration` scenarios; the 12 helper-level `fractional-activity-amounts` scenarios. **Deferred to `rust-ui` as view-layer behavior:** `fractional-activity-amounts` — Default precision is whole numbers, Precision is bounded, Goal fields use the same entry; `activity-filtering` — Default filter on page load, Date Duration dropdown options, Custom Range inputs, Reset on cleared query param, the three Finish-button scenarios, and the six week-loading and scroll-trigger scenarios

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
- [ ] 10.3 Add headless Chrome setup and the `wasm-bindgen-test` tier, with `wasm-bindgen-cli` pinned to the `wasm-bindgen` crate version. Chrome only — no second engine, decided deliberately
- [ ] 10.4 Add `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` for BOTH the host and `wasm32-unknown-unknown` targets — the wasm-gated test module is invisible to a host-only clippy run. Matches the strictness of the existing `CodeAnalysisTreatWarningsAsErrors` setting; both are clean as of section 2
- [ ] 10.5 Confirm the existing .NET build and test steps still run and pass alongside the Rust steps
- [ ] 10.6 Confirm `deploy.yml` is unmodified and the Blazor app still publishes
