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

*Verification:* generate golden `(date, weekKey)` pairs from the running C# implementation across a wide date range (at minimum every day for 20 years, spanning many year boundaries), commit them as a fixture, and assert the Rust implementation reproduces every one.

`GetWeekStartDate`'s linear day-by-day scan from January 1st is also ported as-is, including its silent fallback to January 1st when no date in the year matches the requested key.

### Write a custom serde serializer for timestamps; `chrono`'s RFC 3339 output is wrong for this format

`DateTimeConverter` emits seconds precision with a **non-standard hour-only UTC offset** when the offset's minute component is zero:

```
FormatOffset(-05:00)  →  "-05"      ← not RFC 3339
FormatOffset(+05:30)  →  "+05:30"
FormatOffset(00:00)   →  "Z"
```

`chrono::to_rfc3339` emits `-05:00` unconditionally, so byte-identity breaks on the first timestamp. Reading is equally affected: the C# implementation carries a regex that rewrites `-05` to `-05:00` before parsing, precisely because .NET cannot parse its own output. `chrono`'s parser has the same limitation and needs the same normalization.

So `chrono` is used for calendar arithmetic only. Serialization and deserialization of timestamps go through a hand-written serde adapter that reproduces `Write`, `FormatOffset`, and the read-side regex normalization exactly.

There is a second subtlety: the converter branches on `DateTime.Kind`, emitting `Z` for `Utc` and a local offset otherwise, and on read returns `UtcDateTime` when the parsed offset is zero and local `DateTime` otherwise. Rust has no equivalent of an "unspecified kind" timestamp. The port must model this explicitly — most likely by storing the offset alongside the instant — and the fixture harness is what proves the modeling is right.

### `Storage` trait stays the seam, with `?Send` from the start

WASM futures are not `Send`. The trait is declared `#[async_trait(?Send)]` and that bound propagates through every service. Deciding this at the first trait rather than the fifteenth avoids a miserable retrofit.

Two implementations: `IdbStorage` (gated `#[cfg(target_arch = "wasm32")]`) and `MemStorage` (portable), the latter standing in for Moq so service tests stay native.

### One IndexedDB request-to-future adapter, written once

Every IndexedDB operation is an event-driven `IdbRequest`. A single `request → Future` adapter (`Closure` plus a oneshot channel) is written once and the six storage methods sit on it. `Closure` lifetime management — `.forget()` versus retaining the handle — is the fiddliest code in the change and is deliberately confined to this one module.

### Preserve the object-not-string storage representation

The existing shim `JSON.parse`s on write and `JSON.stringify`s on read, so IndexedDB holds structured-cloned **objects**. The obvious Rust simplification — store the JSON string directly — would make `store.get(key)` return an object where the code expects a string, orphaning every existing user's data. The port keeps the parse/stringify boundary via `js_sys::JSON`.

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

- Which timezone should the golden fixture be generated in? A UTC-only capture would exercise none of the offset formatting paths.
- Does any stored data predate the current `DateTimeConverter`? If an older format exists in long-lived profiles, the read path needs to tolerate it.
- Should `trainer-rs` be a single crate or split (`trainer-core` pure, `trainer-web` browser-facing)? A split enforces the native/browser discipline structurally instead of by review, at the cost of workspace overhead.
