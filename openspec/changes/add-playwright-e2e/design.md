## Context

Trainer is a Blazor WebAssembly **standalone** app (no server) deployed as static files to GitHub Pages under `/trainer/`. `deploy.yml` publishes in Release, rewrites the base href to `/trainer/`, copies `index.html` → `404.html` (Pages' SPA-fallback trick), and rewrites `manifest.json`'s `start_url`.

The `.razor` UI is at 0% coverage. The highest-risk lines are JS interop the browser must actually run — `IndexedDbStorageService`, Chart.js goal charts (`Index`), infinite scroll (`Activities`), and geolocation (`ActivityEntry`, `KnownLocationEntry`). Unit/bUnit tests mock all of this, so the integrated path is unverified. We chose Playwright for real-browser confidence and decided to run it against the actual deployable for maximum fidelity.

## Goals / Non-Goals

**Goals:**
- Real-browser E2E against the **Release artifact**, served the way Pages serves it (`/trainer/` + `404.html` fallback).
- Runs under `dotnet test` locally (zero manual setup) and as a dedicated CI job on PRs.
- Cover the integrated JS-interop / IndexedDB / geolocation flows that unit tests cannot.
- Keep the E2E and deploy publish recipes in sync via a single shared script.

**Non-Goals:**
- Not chasing coverlet %. The run is out-of-process and will not move the line-coverage number.
- No cross-browser matrix initially — Chromium only.
- Not a replacement for the fast unit/helper suite; it is additive.
- Not testing GitHub Pages itself, only an equivalent local serve of the same artifact.

## Decisions

### Decision 1: Serve at the `/trainer/` sub-path with SPA fallback
The static host mounts the published `wwwroot` under `/trainer/` (`UsePathBase("/trainer")` + static files + fallback to `404.html`), and Playwright's `baseURL` is `http://localhost:<port>/trainer/`.
- *Why*: Maximum fidelity. Reproduces the exact base href, asset-path resolution, and deep-link-refresh fallback that only occur on Pages — catching base-href and sub-path asset-404 bugs a root serve would miss.
- *Alternative — serve at root `/`*: trivial with `dotnet serve`, but tests a build that differs from production and can't catch sub-path bugs. Rejected per the fidelity goal.
- *Consequence*: needs a real host (~15 lines) rather than a generic file server; `dotnet serve` can't do path-base + SPA fallback.

### Decision 2: Run against the Release publish artifact
The suite runs against `dotnet publish -c Release` output, not the DevServer.
- *Why*: Exercises trimming/linker output — a WASM-specific failure class invisible to the DevServer and to bUnit. This is the main reason "against the artifact" was chosen.

### Decision 3: Shared publish-and-rewrite script
The publish + base-href/`404.html`/manifest rewrites move into one script (`scripts/publish-pages.*`) that both `deploy.yml` and `e2e.yml` call.
- *Why*: Prevents the E2E environment from drifting from production. Bonus: the deploy recipe itself becomes test-exercised — a broken rewrite fails an E2E test instead of a production push.

### Decision 4: Dual-mode test fixture
An xUnit collection fixture (`IAsyncLifetime`): if `E2E_BASE_URL` is set (CI starts the host as a background step), attach to it; otherwise self-spawn the static host (local dev, zero config). The fixture polls for readiness before any test runs.
- *Why*: One project that "just works" with `dotnet test` locally and stays simple/observable in CI.

### Decision 5: Cold-start safety via explicit ready signals
Tests wait on known post-boot `data-testid` elements, never fixed sleeps or `networkidle` alone. This requires adding `data-testid` anchors to the app shell and key controls.
- *Why*: WASM cold-start (runtime + DLL download before first render) is the #1 source of Blazor E2E flakiness; keying on a real ready signal removes it.

### Decision 6: Per-test isolation + geolocation mocking
Each test gets a fresh Playwright `BrowserContext` (IndexedDB is isolated per context for free). Geolocation flows use `GrantPermissionsAsync(["geolocation"])` + `SetGeolocationAsync(...)`.
- *Why*: Clean DB state per test with no teardown code, and realistic-but-deterministic geolocation — the exact flows that are painful to fake in bUnit.

## Risks / Trade-offs

- **WASM cold-start flakiness** → explicit post-boot waits on `data-testid`, generous navigation timeout, and Playwright trace-on-first-retry. Accepted, mitigated.
- **Coverage number stays flat** → documented in the proposal and README; green E2E + unchanged coverlet % is expected, not a contradiction.
- **Sub-path host is more setup than `dotnet serve`** → accepted (~15-line host) as the cost of the fidelity goal.
- **Publish recipe duplication between deploy and E2E** → eliminated by the shared script (Decision 3).
- **CI time/cost** (Release publish + browser download + WASM boot) → isolated in its own job, Chromium-only, with cached browser binaries; can start non-blocking and be promoted to a required gate once stable.
