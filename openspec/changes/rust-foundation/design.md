## Context

The app is a Blazor WebAssembly PWA, installable and fully offline, deployed to GitHub Pages. All data lives in the user's browser — IndexedDB for activities and activity types, localStorage for in-progress activity timers. There is no server and no backup: if the rewrite reads or writes that data differently, the user's history is gone.

The existing architecture is unusually well-shaped for this port. `IStorageService` is a four-method seam, services are pure over it, and helpers are pure functions. Roughly 900 of the 2,766 test lines mock storage with Moq and would translate to native Rust tests unchanged in spirit.

Two pieces of the existing implementation turned out to be non-obvious on inspection, and both are load-bearing for data compatibility. They are the reason this change exists separately from the UI work.

## Goals / Non-Goals

**Goals:**
- Port domain, helpers, services, and storage to Rust with behavior preserved exactly.
- Prove data compatibility against real exported data before writing the storage implementation.
- Port `Trainer.Tests` scenario-for-scenario, keeping the fast native tier as large as possible.
- Extend CI to run Rust tests alongside the existing .NET tests.
- Leave the Blazor app shipping and unbroken when this change lands.

**Non-Goals:**
- Any UI, any Dioxus dependency, any view code.
- Changes to `deploy.yml`, `service-worker.js`, or the JavaScript shims.
- Deleting any C# code.
- Optimizing bundle size or startup time — explicitly not the motivation for this rewrite.
- Improving or correcting the quirks documented below. Bug-for-bug fidelity is the requirement; user data depends on the current behavior.

## Decisions

### Crate lives at `trainer-rs/`, permanently

A sibling directory to `Trainer/`, mirroring the existing convention. The alternative — a root `Cargo.toml` with `src/` — reads better once the C# is deleted, but it clutters the root during coexistence and promoting the crate later is a disruptive move for no functional gain. Choosing the location once and never moving it is worth a slightly less tidy final layout.

### Split into `trainer-core` and `trainer-web`

This resolves an open question left by the first draft, which proposed a single crate with `#[cfg(target_arch = "wasm32")]` gating.

```
trainer-rs/
  Cargo.toml            workspace, shared dependency versions
  .cargo/config.toml    wasm32 runner = wasm-bindgen-test-runner
  trainer-core/         domain, helpers, services, Storage trait   -> cargo test
  trainer-web/          IdbStorage now, Dioxus views in rust-ui    -> wasm-bindgen-test
  tests/fixtures/       golden fixtures, shared by both tiers
```

The design already identified the risk that "the native tier stays large only if no service reaches for `web_sys` directly — a discipline enforced by review, not by the type system." A workspace split converts that into a compile error: `trainer-core` simply does not depend on `wasm-bindgen`, `js-sys`, or `web-sys`, so browser code cannot creep in unnoticed and quietly shrink the fast test tier.

The cost is two extra manifests. Given that the native/browser ratio is the single property most likely to erode over a long port, having the compiler defend it is worth the ceremony.

Fixture-loading helpers live behind a `test-support` feature on `trainer-core`, which `trainer-web` enables as a dev-dependency, so both tiers assert against the same files without the helpers shipping in the library.

### Port `WeekHelper`'s algorithm literally; do NOT use `chrono::IsoWeek`

**This is the single most important decision in the change.**

`WeekHelper` documents itself as producing "ISO 8601 week format (YYYY.WW)". It does not. It computes:

```
year = InvariantCulture.Calendar.GetYear(dateTime)          // calendar year
week = GetWeekOfYear(dateTime, FirstFourDayWeek, Monday)    // .NET week-of-year
key  = $"{year}.{week:D2}"
```

Real ISO 8601 uses the **ISO week-year**, which diverges from the calendar year at year boundaries. .NET's `FirstFourDayWeek` rule can return week 53 for an early-January date while `GetYear` returns the new calendar year — producing a key like `2026.53`, a bucket that proper ISO numbering would call `2025.53`.

`chrono`'s `iso_week()` returns correct ISO week-years. Using it would **silently re-bucket every activity near a year boundary**, orphaning that data in storage keys nothing reads. The port must reimplement .NET's `FirstFourDayWeek` / Monday week-of-year rule and pair it with the plain calendar year.

*Alternative considered:* correct the algorithm to true ISO weeks and migrate existing buckets. Rejected — it converts a zero-risk port into a data migration, for a defect no user has ever perceived.

*Verification:* golden `(date, weekKey)` pairs generated from the C# implementation across 2010–2040 (11,323 days) are committed as `tests/fixtures/week-keys.csv`, and the Rust implementation must reproduce every one.

**Confirmed by the generated fixtures.** The divergence is real and appears every year. January 1st routinely lands in week 52 or 53 *of the new calendar year* — `2010.53`, `2011.52`, `2016.53`, `2021.53`, `2027.53` — where true ISO numbering would assign it to the previous week-year. `chrono::IsoWeek` would produce a different key for every one of these dates.

There is no bucket collision: the calendar week straddling New Year is *split* across two buckets (`2025.53` holds Dec 29–31, `2026.01` holds Jan 1–4). Storage is self-consistent as long as reads and writes both go through `GetWeekKey`.

### `GetWeekStartDate` returns a date outside its own bucket — do NOT port this as-is

This corrects an earlier draft of this document, which described the defect as "a silent fallback to January 1st when no date in the year matches." That is not what happens. The linear scan always finds a match; the defect is what it does next.

For the first week key of any year, the scan finds January 1st and then walks back to that week's Monday — which lies in the *previous* year, in a *different* bucket:

```
GetWeekStartDate("2026.01") → 2025-12-29    (bucket 2025.53, not 2026.01)
GetWeekEndDate("2026.01")   → 2026-01-04
```

`tests/fixtures/week-key-anomalies.csv` records this for all 26 year boundaries in range — one per year, without exception.

This is load-bearing. Both `Activities.razor` and `Calendar.razor` use the pattern:

```csharp
var weekStart = WeekHelper.GetWeekStartDate(weekKey);
var weekEnd   = WeekHelper.GetWeekEndDate(weekKey);
var activities = await ActivityService.GetAllAsync(weekStart, weekEnd);
```

and `ActivityService.GetAllAsync(start, end)` performs **no date filtering** — it resolves the range to week keys and returns those buckets whole. So asking for week `2026.01` loads both `2025.53` and `2026.01` in full, while the caller records only `2026.01` in `loadedWeekKeys`. When the loop later reaches `2025.53`, it loads it a second time and every activity in it is appended twice.

**The bug-for-bug rule does not extend here.** The rule exists because stored bytes depend on `GetWeekKey`; nothing is persisted from `GetWeekStartDate`, so fixing it changes no data and breaks no compatibility. Faithfully reproducing a duplicate-rendering defect would be fidelity for its own sake.

Because this is a user-visible behavior change rather than a port, the fix is deliberately **not** taken in this change. `GetWeekStartDate` is ported as-is with the golden values in `tests/fixtures/week-boundaries.csv`, the defect is reproduced under test so the Rust behavior is pinned and known, and correcting it is left to its own change against the shipping C# app or the ported Rust app.

### Write a custom serde serializer for timestamps; `chrono`'s RFC 3339 output is wrong for this format

`DateTimeConverter` emits seconds precision with a **non-standard hour-only UTC offset** when the offset's minute component is zero:

```
FormatOffset(-05:00)  →  "-05"      ← not RFC 3339
FormatOffset(+05:30)  →  "+05:30"
FormatOffset(00:00)   →  "Z"
```

`chrono::to_rfc3339` emits `-05:00` unconditionally, so byte-identity breaks on the first timestamp. Reading is equally affected: the C# implementation carries a regex that rewrites `-05` to `-05:00` before parsing, precisely because .NET cannot parse its own output. `chrono`'s parser has the same limitation and needs the same normalization.

So `chrono` is used for calendar arithmetic only. Serialization and deserialization of timestamps go through a hand-written serde adapter that reproduces `Write`, `FormatOffset`, and the read-side regex normalization exactly.

### Timestamp representation: retain the offset, unlike the C# reader

This resolves task 3.3. `DateTimeConverter` branches on `DateTime.Kind`, and the C# reader returns `dto.UtcDateTime` when the parsed offset is zero and `dto.DateTime` otherwise. That second branch **discards the offset**, yielding a wall clock with kind `Unspecified`; a later write then re-derives an offset from whatever `TimeZoneInfo.Local` says at that moment.

The consequence was measured rather than assumed. Reading the Los Angeles fixture under `TZ=Asia/Kolkata` and re-serializing gives:

```
written in LA            re-saved in Kolkata        re-saved in UTC
2026-01-01T08:56:44-08   2026-01-01T08:56:44+05:30  2026-01-01T08:56:44Z
2026-07-04T12:30:15-07   2026-07-04T12:30:15+05:30  2026-07-04T12:30:15Z
2026-06-15T10:00:00Z     2026-06-15T10:00:00Z       2026-06-15T10:00:00Z
```

The wall clock survives; the instant moves by 13.5 hours. Only `Z` values are stable, because a zero offset reads back as kind `Utc`. In the shipping app this means a user who travels and re-saves an activity silently re-anchors it to their new timezone. Recorded in `timestamps-crosszone-*`.

**Decision: keep the offset.** The Rust type mirrors the two states observable on the wire rather than .NET's three kinds:

```rust
enum TrainerTime {
    Utc(NaiveDateTime),                                    // -> "…Z"
    Offset { naive: NaiveDateTime, offset: FixedOffset },  // -> "…-08", "…+05:30"
}
```

This is a **deliberate divergence** from the bug-for-bug rule, taken because:

- It is byte-identical for every value the C# implementation ever wrote, so no stored data changes and nothing is orphaned.
- It diverges only in the travel case, where the C# behavior is itself the defect.
- Nothing user-visible changes: the app displays the wall clock and `WeekHelper` buckets on the wall clock, and both are preserved either way. The divergence is confined to which offset string is emitted on re-serialization.
- It makes serialization **pure**. Reproducing the C# behavior would require reading the ambient timezone during serialization, which would drag `js-sys` into `trainer-core` and break the crate split — or force an injected timezone provider threaded through every call. Retaining the offset means all six timezone fixtures are asserted natively, with no `TZ` manipulation in the test suite.

Resolving an offset for a *new* timestamp still needs the ambient zone, but that happens at construction in `trainer-web`, not during serialization.

A null or empty `when` maps to `default(DateTime)` (`0001-01-01T00:00:00`) as the C# reader does. That path is unreachable for real data, since `Activity.When` is non-nullable and the writer always emits a value.

**Confirmed against real data.** All 527 timestamps in the real export use hour-only offsets (`-08` and `-07`), never `-08:00`. Because that export is Pacific-only, the remaining branches were captured by driving the real `DateTimeConverter` under other `TZ` values, which .NET honors on Unix: `Asia/Kolkata` for `+05:30`, `Australia/Eucla` for the 45-minute `+08:45`, and `UTC` for `Z`. Note that under a zero-offset zone even `DateTimeKind.Local` values emit `Z`, since `FormatOffset(TimeSpan.Zero)` returns `"Z"` before the local branch can apply.

### Two serde configurations, not one

The export format and the storage format are **not the same format**, which an earlier draft of this document assumed.

| | `DefaultIgnoreCondition` | Unset optional field |
|---|---|---|
| `ExportImportService` | `WhenWritingNull` | omitted |
| `IndexedDbStorageService` | *(unset)* | `"durationSeconds":null` |

Both are captured as fixtures (`timestamps-export-*`, `timestamps-storage-*`). The port needs both configurations, and the export fixture alone cannot validate the storage path — which is why the raw IndexedDB dump remains a required task rather than a nice-to-have.

### Reproduce `System.Text.Json`'s string escaping

Found while byte-comparing the Kathmandu fixture, and not anticipated by any earlier draft. The app sets no custom `Encoder`, so both serializer configurations use `JavaScriptEncoder.Default`, which escapes far more than JSON requires as XSS defence-in-depth. `serde_json` escapes only the minimum.

The escape set was measured from the C# implementation across all 128 ASCII code points plus a non-ASCII spread, and committed as `tests/fixtures/json-escaping.json`:

| input | `System.Text.Json` | `serde_json` |
|---|---|---|
| `"` | `"` | `\"` |
| `&` `'` `+` `<` `>` `` ` `` | `&` … | literal |
| U+007F and above | `\uXXXX`, surrogate pairs above the BMP | literal UTF-8 |
| other C0 controls | `` (uppercase) | `` (lowercase) |
| `/` `\` | unescaped / `\\` | same |

This is not a corner case. The real export contains 15 escapes — `½` (½), `'`, `+`, `&` — all from note text. And **every user east of Greenwich has `+` in every stored timestamp**, since a positive UTC offset escapes: `+05:45` is written `+05:45`.

Implemented as a `serde_json::ser::Formatter` rather than a custom serializer: `write_string_fragment` catches the characters `serde_json` considers safe, and `write_char_escape` corrects the two where the two libraries disagree. Numbers and structure still go through `serde_json` unchanged.

Escaping matters only where the JSON string is itself the artifact — exports. Storage values are handed to `JSON.parse` and stored as structured-cloned objects, so escaping is decoded before it reaches IndexedDB. Matching in both formats is nonetheless simpler and harmless.

*Fixture consequence:* the first de-identification pass replaced note text with plain ASCII and destroyed all escape evidence, so `export.json` initially exercised none of this. `deidentify.py` now emits .NET-escaped JSON and carries escape-triggering characters through into replacement text, so the fixture tests the escaper without carrying real content.

### `EmptyStringAsNullConverter` is dead code

`Trainer/Serialization/EmptyStringAsNullConverter.cs` converts empty strings to null on write and null to empty string on read. It is referenced nowhere: not registered in either `JsonSerializerOptions`, not applied via `[JsonConverter]`. Real exports therefore contain `"notes":""` rather than omitting the field. It is not ported.

### `KnownLocationService.AssignId` is not reproducible, by design

```csharp
int candidate = HashCode.Combine(latitude.GetHashCode(), longitude.GetHashCode());
```

`System.HashCode` is seeded with a per-process random value, and .NET documents its output as unstable across executions. The same coordinates saved in two sessions yield different ids — which is why the real export contains large ids of both signs (`-2140118897` through `1547621909`).

There is consequently no algorithm to port. The requirement is only that stored ids are preserved verbatim and never regenerated; new ids may be produced by any collision-avoiding scheme. Attempting fidelity here would be imitating randomness.

### `Storage` trait stays the seam, with `?Send` from the start

WASM futures are not `Send`. The trait is declared `#[async_trait(?Send)]` and that bound propagates through every service. Deciding this at the first trait rather than the fifteenth avoids a miserable retrofit.

Two implementations: `IdbStorage` (gated `#[cfg(target_arch = "wasm32")]`) and `MemStorage` (portable), the latter standing in for Moq so service tests stay native.

### One IndexedDB request-to-future adapter, written once

Every IndexedDB operation is an event-driven `IdbRequest`. A single `request → Future` adapter (`Closure` plus a oneshot channel) is written once and the six storage methods sit on it. `Closure` lifetime management — `.forget()` versus retaining the handle — is the fiddliest code in the change and is deliberately confined to this one module.

### Preserve the object-not-string storage representation

The existing shim `JSON.parse`s on write and `JSON.stringify`s on read, so IndexedDB holds structured-cloned **objects**. The obvious Rust simplification — store the JSON string directly — would make `store.get(key)` return an object where the code expects a string, orphaning every existing user's data. The port keeps the parse/stringify boundary via `js_sys::JSON`.

**Confirmed against a real profile.** A raw dump of database `Trainer` (version 1, object store `activities`, 34 keys) reports `typeof === "object"` with `constructor === "Array"` for 33 entries. Not strings.

The 34th entry is the one the design missed: **`activityNextId` is stored as a bare JavaScript `Number`** (value 536), because `SetItemAsync<int>` serializes to `"536"` and the shim's `JSON.parse` yields a primitive. The storage layer must therefore handle scalar values, not only objects and arrays. Four key shapes exist in practice:

| Key | Stored type |
|---|---|
| `activities-{weekKey}` | Array |
| `activityTypes` | Array |
| `knownLocations` | Array |
| `activityNextId` | Number |

The dump was taken with `indexedDB.open('Trainer')` and no explicit version, so no upgrade transaction could fire. The port should open the same way.

### `notes` has three distinct states, and they must not be collapsed

Real profile counts, identical across both formats:

| State | Storage | Export | Count |
|---|---|---|---|
| `None` | `"notes":null` | omitted | 50 |
| `Some("")` | `"notes":""` | `"notes":""` | 38 |
| `Some(text)` | `"notes":"…"` | `"notes":"…"` | 439 |

An `Option<String>` that treats the empty string as absent — a natural-looking simplification, and the one `EmptyStringAsNullConverter` would have introduced had it ever been wired up — silently corrupts 38 activities. This is the second reason that converter must stay unported.

Storage additionally writes every optional field explicitly: `durationSeconds` is null 439 times and `knownLocationId` 328 times, where the export omits them entirely.

### The migration and active-activity paths have no real-data coverage

The captured profile's `localStorage` is **empty** — no legacy `activities` or `activityTypes` keys, and no `trainer_active_activities`. So neither the one-time localStorage-to-IndexedDB migration (task 7.6) nor the active-activity persistence format (task 8.5) can be validated against real data. Both need synthetic fixtures built from the C# implementation, and both should be treated as lower-confidence than the paths backed by a real capture.

Note that roughly a third of `indexeddb-storage.js` is Blazor interop scar tissue (`getItems` defensively handling arrays that Blazor may have marshalled as JSON strings). That code has no Rust counterpart and is simply dropped.

### Test tiers split by what needs a browser

| Tier | Runner | Covers |
|---|---|---|
| Native | `cargo test` | models, serde, week keys, all six services against `MemStorage`, search filter, duration parsing, decimal amounts, formatting |
| Browser | `wasm-bindgen-test` + headless Chrome | `IdbStorage`, and in `rust-ui`, geolocation / notifications / IntersectionObserver |

The native tier stays large only if no service reaches for `web_sys` directly. That is a discipline enforced by review, not by the type system.

`Trainer.Tests` is ported scenario-for-scenario rather than reimagined, using the nine existing capability specs plus the new `storage-data-compatibility` spec as the checklist. Coverage is not gated in CI today (`test.yml` runs plain `dotnet test` despite `coverlet.collector` being referenced), so no coverage gate is introduced here.

## Risks / Trade-offs

**Week key divergence silently orphans data** → Golden-value fixture across 20 years of dates, generated from the C# implementation, asserted in the native test tier. This lands before `IdbStorage` is written.

**Timestamp format divergence breaks byte-identity** → Round-trip fixture assertion on a real export; custom serde adapter rather than `chrono`'s RFC 3339 helpers.

**`DateTime.Kind` has no Rust equivalent, so the local/UTC modeling could be subtly wrong** → The fixture must include timestamps of both kinds, ideally captured from a profile in a non-UTC timezone with a non-zero-minute offset available for the `+05:30` path.

**The fixture is only as good as the data in it** → Curate deliberately: activities near year boundaries, fractional amounts, private types, durations, known locations, empty and null notes, and an in-progress activity. A fixture of ten ordinary rows proves almost nothing.

**CI gets slower** → `Swatinem/rust-cache` plus a pinned `wasm-bindgen-cli` matching the `wasm-bindgen` crate version. During this change CI runs both .NET and Rust suites; `rust-ui` removes the .NET half.

**Bug-for-bug fidelity preserves real defects** → Accepted deliberately. The week-key and offset quirks are recorded here so that a later change can correct them as an intentional, migrated decision rather than an accident.

## Migration Plan

No user-facing deployment. `deploy.yml` is untouched and the Blazor app continues to ship. The rollback for this change is deleting `trainer-rs/` and reverting `test.yml`.

The user-facing migration — service worker cutover for installed PWAs — belongs entirely to `rust-ui`.

## Open Questions

**Resolved**

- *Which timezone for the golden fixture?* The real capture is Pacific-only (`-08`/`-07`). The other branches were generated by driving the real converter under `TZ=Asia/Kolkata` (`+05:30`), `TZ=Australia/Eucla` (`+08:45`), and `TZ=UTC` (`Z`).
- *Single crate or split?* Split — see Decisions.
- *Does stored data predate the current `DateTimeConverter`?* All 527 activities in the captured profile use the current format, so this is low risk. Not proof for every profile, but no longer a live concern.

**Still open**

- The migration and active-activity paths have no real-data coverage (empty `localStorage` in the capture), so their fixtures are synthetic and their confidence is lower than everything else.
- Should the browser tier run against a second engine? The port currently targets headless Chrome only; IndexedDB behavior differs subtly across engines, and the app is explicitly installable on iOS where Safari is the only option.
