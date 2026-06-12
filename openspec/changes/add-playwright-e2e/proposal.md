## Why

The `.razor` UI layer is at **0% test coverage**, and roughly half of its lines are JS-interop heavy — IndexedDB persistence, Chart.js goal charts, infinite scroll, and geolocation capture. Unit tests (and even bUnit) can only *mock* that interop, so the integrated behavior that actually ships is unverified. We want real confidence that the app works in a browser, exercised against the artifact we actually deploy.

Playwright drives a real browser, runs the real WASM runtime, uses real IndexedDB, and can grant/mock geolocation — covering exactly the surface unit tests cannot. Running it against the **Release publish artifact served at the `/trainer/` sub-path** (the way GitHub Pages serves it) additionally catches trimming/linker and base-href/routing failures that only appear in production.

**Coverage caveat (set expectations):** this suite runs out-of-process against the WASM runtime, so it will **not** move the coverlet line-coverage number. Its value is integration confidence, not a higher percentage.

## What Changes

- Add a new `Trainer.E2E` xUnit project using `Microsoft.Playwright` (Chromium), runnable via `dotnet test`.
- Run the suite against the **Release publish artifact** served at the **`/trainer/` sub-path** with the same SPA fallback (`404.html`) behavior as GitHub Pages — maximum production fidelity.
- Extract the deploy publish-and-rewrite steps (base href, `404.html`, manifest) into a **shared script** consumed by both `deploy.yml` and the E2E job, so the tested artifact cannot drift from the deployed one.
- Add a minimal **static host** that serves the published `wwwroot` under `/trainer/` with correct `.wasm` MIME and the SPA fallback.
- Add a **dual-mode test fixture**: attach to `E2E_BASE_URL` when set (CI), otherwise self-spawn the host (local dev, zero config).
- Add `data-testid` anchors to the app shell and key controls so tests can wait on real post-boot signals (cold-start safety).
- Cover the first integrated smoke flows: log-activity end-to-end, IndexedDB persistence round-trip, geolocation capture, goal chart render, active-activity start/stop, export/import.
- Add a dedicated `e2e.yml` GitHub Actions job that publishes, installs the browser, serves the artifact, runs the suite, and uploads Playwright traces on failure.

## Capabilities

### New Capabilities
- `end-to-end-testing`: A browser-driven E2E suite that runs against the Release deployment artifact at the `/trainer/` sub-path, executes via `dotnet test` and a dedicated CI job, isolates tests per browser context, and covers the JS-interop / IndexedDB / geolocation flows that unit tests cannot exercise.

### Modified Capabilities
<!-- None. This adds test infrastructure; no product behavior changes. -->

## Impact

- **New project**: `Trainer.E2E` (xUnit + Microsoft.Playwright); added to `Trainer.sln`. Excluded from the fast unit-test run.
- **Trainer source**: `data-testid` attributes on the app shell and key controls — markup-only, no behavior change.
- **CI**: new `.github/workflows/e2e.yml`; `deploy.yml` refactored to call the shared publish script (no deploy behavior change).
- **Tooling**: a small static host project/script and a shared publish script under `scripts/`.
- **Not affected**: coverlet line coverage (out-of-process browser run); the fast `Trainer.Tests` suite and its CI job; product behavior.
