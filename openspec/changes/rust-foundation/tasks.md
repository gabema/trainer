## 0. Notes

- [x] 10.7 Add a CI guard asserting no test run modifies a committed fixture. Golden data that a test can rewrite would make the suite self-confirming

## 1. Golden fixtures (before any implementation)

- [x] 1.1 Add a temporary C# console/test harness that dumps `(date, weekKey)` pairs from `WeekHelper.GetWeekKey` for every day across a 20-year span covering multiple year boundaries; commit the output as `trainer-rs/tests/fixtures/week-keys.csv`
- [x] 1.1a Dump `GetWeekStartDate` / `GetWeekEndDate` golden values as `week-boundaries.csv` — needed by tasks 5.3 and 5.4, which run after the harness is deleted at 1.5
- [x] 1.1b Dump the year-boundary round-trip failures as `week-key-anomalies.csv` so the defect is pinned rather than rediscovered during the port
- [x] 1.2 De-identify a real export into `trainer-rs/tests/fixtures/export.json` via `deidentify.py`, preserving every structural property (527 activities, 8 field-presence combinations, both hour-only offsets, netBenefit 0/1/2, decimalPlaces 0/2, both `isPrivate` values, signed knownLocation ids, the `2026.01` boundary bucket) while replacing names, units, note text, and coordinates. Real personal data is not committed
- [x] 1.3 Generate timestamp fixtures by driving the real `DateTimeConverter` under different `TZ` values, covering all three offset-formatting branches: hour-only (`-08`/`-07`), non-zero-minute (`+05:30` Kolkata, `+08:45` Eucla), and zero (`Z`)
- [x] 1.3a Capture both serializer configurations separately — `timestamps-export-*` (nulls omitted) and `timestamps-storage-*` (nulls written) — because `ExportImportService` sets `DefaultIgnoreCondition` and `IndexedDbStorageService` does not
- [x] 1.3b Capture `timestamps-roundtrip-*` recording what each emitted string parses back to through the converter's own `Read` path
- [x] 1.4 De-identify a raw IndexedDB dump into `trainer-rs/tests/fixtures/idb-snapshot.json`. Confirms values are structured-cloned Arrays (33 entries), that `activityNextId` is a bare JS Number, and that storage writes every optional field explicitly as `null`
- [x] 1.4a Synthesize fixtures for the legacy localStorage migration and for `trainer_active_activities` by driving both real C# code paths with a mocked `IJSRuntime`, since the captured profile's localStorage was empty
- [x] 1.4b Record `active-activities.json`: the write format, the read-back with `Kind` preserved, and that the key is removed rather than emptied
- [x] 1.4c Record `legacy-migration.json`: a legacy flat list splitting into week buckets across a year boundary, `activityTypes` written unbucketed, and both legacy keys removed
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

- [x] 7.1 Define the `Storage` trait as `#[async_trait(?Send)]`. It deals in JSON strings rather than mirroring the C# generic `GetItemAsync<T>`, which is not object-safe in Rust and was only ever sugar over the same string boundary
- [x] 7.2 Implement portable `MemStorage` to stand in for Moq, plus a dependency-free `block_on` for the native tier that panics rather than spins if a future ever yields
- [x] 7.3 Implement `await_request`, wrapping an `IdbRequest` in a `Promise` so `JsFuture` can await it. Handlers use `Closure::once_into_js`; the unfired sibling leaks a small allocation, deliberately, because dropping a `Closure` from inside its own callback would be unsound
- [x] 7.4 Implement `IdbStorage` against database `Trainer` v1, object store `activities`, preserving the `js_sys::JSON` parse-on-write / stringify-on-read boundary. Open **at version 1 with an upgrade handler**, correcting this task as written: opening without a version leaves a fresh profile with no object store, and version 1 against an existing version-1 database never fires the upgrade anyway
- [x] 7.4a Cover all four storage key shapes seen in the real profile: `activities-{weekKey}` (Array), `activityTypes` (Array), `knownLocations` (Array), `activityNextId` (Number)
- [x] 7.5 Implement week bucketing as a `WeekBucketed<S>` decorator that itself implements `Storage`, reproducing the `activities` aggregate key and removing emptied buckets. Being a decorator over `MemStorage` keeps it in the native tier
- [x] 7.6 Implement the one-time localStorage-to-IndexedDB migration, non-fatal on failure, verified byte-identical against the C#-captured `legacy-migration.json`
- [x] 7.7 Verify `IdbStorage` against `idb-snapshot.json` under `wasm-bindgen-test`: buckets read back as `Array`, `activityNextId` as a number, the database is not upgraded, and prefix search excludes the sibling `activit*` keys
- [x] 7.8 Port `LocalStorageServiceTests` scenario-for-scenario against `MemStorage`, and add a `LocalStorage` implementation for the browser

- [x] 7.9 Reproduce .NET's whole-double formatting (`10.0` -> `10`) via `write_f64`, and regenerate the fixtures, which had carried Python's `repr` form and so never matched C#

## 8. Services

- [x] 8.1 Port `ActivityService` including the next-id counter, week-transition handling on update, and `RecalculateNextIdAsync`. Preserves the asymmetry where `update` deletes an emptied bucket but `delete` leaves it as `[]`, and where a date range returns whole buckets with no filtering
- [x] 8.2 Port `ActivityTypeService`, keeping reads sorted by name while writes preserve storage order
- [x] 8.3 Port `GoalService`, covering the `neutral-benefit` spec scenarios and the four-week weekly-then-daily fallback
- [x] 8.4 Port `KnownLocationService` including `FindNearbyAsync` (Haversine, 100m) and `NextAutoNameAsync`. `AssignId` is NOT reproduced — `HashCode.Combine` is randomly seeded per process, so there is nothing stable to port; new ids use deterministic FNV-1a over the coordinates with collision increment, and stored ids are preserved verbatim
- [x] 8.5 Port `ActiveActivityService`, verified to write a payload byte-identical to the recorded C# output. The key is removed rather than emptied, corrupt state is discarded silently, and the event/tick machinery becomes a `version()` counter since ticking is a view concern
- [x] 8.5a Implement `ActiveTime`, the third wire format: full `±hh:mm` offsets, an offset-less form RFC 3339 rejects, and sub-second precision trimmed at .NET's 100ns tick resolution. Asserted against `active-activities.json`
- [x] 8.6 Port `ExportImportService`, verified by importing the real 527-activity export and re-exporting it byte-identically. Accepts the legacy flat array and PascalCase names. **Diverges deliberately**: every section is deserialized before the store is cleared, because the C# clears first and so destroys all data when a structurally valid file fails to deserialize
- [x] 8.7 Port `WeekFillLoader`, including the guarantee that the caller records the loaded key so the loop terminates
- [x] 8.8 Port `ActivityServiceTests`, `ActivityTypeServiceTests`, `GoalServiceTests`, `KnownLocationServiceTests`, `ActiveActivityServiceTests`, `ExportImportServiceTests` and `WeekFillLoaderTests` scenario-for-scenario against `MemStorage`

## 9. Cross-implementation compatibility

- [x] 9.1 Generate `csharp-export.json` by driving the real `ExportImportService` over the real `IndexedDbStorageService`, then assert the Rust implementation imports and re-exports it **byte-identically**. Unlike `export.json`, its bytes come start-to-finish from the C# export path
- [x] 9.2 Commit `rust-export.json`, produced by the Rust serializer and guarded against staleness, and add `RustInteropTests` on the C# side that imports it and asserts ids, all three notes states, durations, coordinates, week bucketing, `DateTimeKind`, and escaped characters all survive
- [x] 9.3 Load the shipping `indexeddb-storage.js` verbatim into the browser tier — with its `DB_NAME` rewritten for isolation — and assert both directions: the shim reads what Rust wrote (arrays and the bare-number scalar), Rust reads what the shim wrote, and both agree on prefix search
- [x] 9.4 Walk all nine capability specs — 41 requirements, 134 scenarios — and record the result in `spec-coverage.md`: 20 requirements covered here, 21 deferred to `rust-ui` with an identified view-layer reason each. Nothing unaccounted for

## 10. CI

- [x] 10.1 Add a separate `rust` job with `dtolnay/rust-toolchain`, the `wasm32-unknown-unknown` target, and `Swatinem/rust-cache`. Two jobs rather than more steps in one, so the Rust and .NET suites run in parallel and fail independently
- [x] 10.2 Add `cargo test -p trainer-core` for the native tier
- [x] 10.3 Add the `wasm-bindgen-test` tier against headless Chrome. `wasm-bindgen-cli` is pinned by **reading the version out of `Cargo.lock`** rather than hardcoding it, so a dependency bump cannot desync the CLI from the crate; the install is skipped when the cached binary already matches. Chrome only, decided deliberately
- [x] 10.4 Add `cargo fmt --check` plus `cargo clippy --all-targets -- -D warnings` for BOTH targets, matching the strictness of `CodeAnalysisTreatWarningsAsErrors`. The host-only run cannot see `trainer-web`'s wasm-gated code or any of its tests
- [x] 10.5 The .NET job is unchanged and still gates merges; its 223 tests pass, now including `RustInteropTests`, so the `dotnet` job also guards the Rust-to-C# direction
- [x] 10.6 `deploy.yml` is byte-identical to `main`, and `dotnet publish --configuration Release` still succeeds, so the shipping app is verifiably unbroken rather than merely untouched
