## Why

The app is being moved off C# / .NET to Rust. Blazor WebAssembly works, but the team wants the codebase in Rust — the motivation is language choice, not bundle size or performance.

A rewrite of a shipping, offline-first PWA carries one dominant risk: existing users have real activity history sitting in their browser's IndexedDB, written by the C# serializer in a specific shape. If the Rust implementation reads or writes that data even slightly differently, installed users silently lose their history with no server-side backup to recover from.

This change therefore ports the non-visual half of the app — domain, services, storage, tests — and proves data compatibility against real exported data, *before* any UI work begins. The Blazor app continues to build and deploy unchanged throughout. A follow-up change (`rust-ui`) replaces the views and retires the C# project.

## What Changes

- Add a Rust crate targeting `wasm32-unknown-unknown` alongside the existing C# project. Both coexist for the duration of this change.
- Port the domain models — `Activity`, `ActivityType`, `KnownLocation`, `NetBenefit`, `DurationOption` — to Rust types with `serde`, producing JSON byte-identical to `System.Text.Json` with camelCase naming and the custom `DateTimeConverter`.
- Port the pure helpers: `WeekHelper`, `DateTimeHelper`, `DecimalAmount`, `DurationInput`, `ActivitySearchFilter`, `ActivityAmountDisplay`.
- Port the services — activity, activity type, goal, known location, active activity, export/import — behind an async `Storage` trait, preserving the existing `IStorageService` seam so services stay testable without a browser.
- Implement IndexedDB storage natively via `web-sys`, **preserving the current on-disk representation**: values are stored as structured-cloned JS objects (the JS shim `JSON.parse`s on write and `JSON.stringify`s on read), not as strings. Week bucketing under `activities-{weekKey}` is unchanged.
- Add a fixture-based data compatibility harness: a real export from the running Blazor app is committed as a test fixture and asserted to round-trip byte-identically through the Rust models.
- Port `Trainer.Tests` scenario-for-scenario. Pure logic runs natively under `cargo test`; storage code runs under `wasm-bindgen-test` in headless Chrome.
- Update `.github/workflows/test.yml` to run the Rust test tiers **in addition to** the existing .NET tests, with Rust build caching and a pinned `wasm-bindgen-cli`.

Explicitly **not** in this change: any UI, any Dioxus dependency, any change to `deploy.yml`, any change to the service worker, and any deletion of C# code. The Blazor app remains the shipping app when this change lands.

## Capabilities

### New Capabilities

- `storage-data-compatibility`: Guarantees that activity data written by the existing C# implementation remains readable and writable by the Rust implementation, that the IndexedDB representation and week-bucket key format are preserved exactly, and that export/import round-trips without loss across implementations.

### Modified Capabilities

None. All nine existing capability specs describe behavior that this change preserves rather than alters — they serve as the parity checklist for the port, not as deltas.

## Impact

**Added**
- New Rust crate: `serde`, `serde_json`, `chrono`, `wasm-bindgen`, `js-sys`, `web-sys` (feature-gated: IndexedDB), `wasm-bindgen-futures`, `async-trait`, `wasm-bindgen-test`.
- Test fixture containing a real data export.
- `.github/workflows/test.yml`: Rust toolchain, `wasm32-unknown-unknown` target, `Swatinem/rust-cache`, headless Chrome, pinned `wasm-bindgen-cli`.

**Unchanged (deliberately)**
- `Trainer/` C# project — still builds, still deploys.
- `.github/workflows/deploy.yml`.
- `Trainer/wwwroot/js/*` and `service-worker.js` — the Rust IndexedDB implementation is written against the same storage format, but the JS shims stay in place serving the Blazor app until `rust-ui`.
- `openspec/config.yaml` — its tech-stack context still describes the shipping app; it is rewritten in `rust-ui`.

**Risks**
- **Data loss for existing installed users** if the IndexedDB representation diverges. Mitigated by the fixture harness landing before the storage implementation.
- C# `DateTime` local/UTC semantics are implicit and the current `DateTimeConverter` encodes whatever the existing behavior is, including any quirks that stored data now depends on. `chrono` forces explicitness, which is where a silent behavior change is most likely to enter.
- CI runtime grows: cold Rust builds are minutes where `dotnet restore` was seconds, and a headless-browser tier is added.
