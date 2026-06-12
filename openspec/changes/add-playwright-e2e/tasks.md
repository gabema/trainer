## 1. E2E project scaffold

- [ ] 1.1 Create `Trainer.E2E` xUnit project (`net10.0`) referencing `Microsoft.Playwright`; add it to `Trainer.sln`
- [ ] 1.2 Keep it out of the fast unit run — the existing `Trainer.Tests` CI job continues to target only `Trainer.Tests`
- [ ] 1.3 Document the one-time browser bootstrap (`pwsh bin/Debug/net10.0/playwright.ps1 install chromium`)

## 2. Shared publish recipe

- [ ] 2.1 Extract the publish + base-href / `404.html` / `manifest.json` rewrites from `deploy.yml` into a single shared script (`scripts/publish-pages.*`) parameterized by output path
- [ ] 2.2 Update `deploy.yml` to call the shared script; confirm the deploy output is byte-equivalent to today (no deploy behavior change)

## 3. Sub-path static host

- [ ] 3.1 Add a minimal ASP.NET static host that serves the published `wwwroot` under `/trainer/` (`UsePathBase` + static files + correct `.wasm`/`.dll` MIME)
- [ ] 3.2 Add SPA fallback: unknown routes under `/trainer/` return `404.html` (mirrors Pages)
- [ ] 3.3 Bind to an ephemeral/configurable port and expose it so the fixture can discover the served URL

## 4. Test fixture & selectors

- [ ] 4.1 Dual-mode collection fixture (`IAsyncLifetime`): use `E2E_BASE_URL` when set, else publish + self-spawn the host; poll for readiness before tests
- [ ] 4.2 Base test class: fresh `BrowserContext` per test, `baseURL` = `.../trainer/`, and a shared "wait for app boot" helper keyed on a `data-testid`
- [ ] 4.3 Add `data-testid` anchors to the app shell and key controls in `Trainer/` (markup-only, no behavior change)

## 5. Smoke flows

- [ ] 5.1 Log-activity end-to-end: create an activity type → log an activity → it appears on Activities and Calendar
- [ ] 5.2 IndexedDB persistence round-trip: log an activity → reload the page → it is still present
- [ ] 5.3 Geolocation capture: grant permission + set coordinates → a location-capture flow records them (`KnownLocationEntry` / `ActivityEntry`)
- [ ] 5.4 Goal chart: with data present, the `Index` goal chart renders
- [ ] 5.5 Active activity: start → `ActiveActivityNotification` appears → stop
- [ ] 5.6 Export/import: export data → re-import → round-trips

## 6. CI

- [ ] 6.1 Add `.github/workflows/e2e.yml`: checkout, setup-dotnet, run the shared publish script, `playwright install --with-deps chromium`, start the host (background, set `E2E_BASE_URL`), `dotnet test Trainer.E2E`
- [ ] 6.2 Cache Playwright browser binaries between runs
- [ ] 6.3 Configure Playwright `trace: on-first-retry`; upload traces/screenshots as artifacts on failure
- [ ] 6.4 Start the job non-blocking (its own workflow); promote to a required check once it is stable

## 7. Docs

- [ ] 7.1 README: how to run the E2E suite locally (and the one-time browser install), plus the coverage caveat — E2E gives integration confidence and does **not** move the coverlet line-coverage number

## 8. Verify

- [ ] 8.1 Run `dotnet test Trainer.E2E` locally with no `E2E_BASE_URL` and confirm the suite publishes, serves at `/trainer/`, and all smoke flows pass
- [ ] 8.2 Confirm a deep-link refresh under `/trainer/` (e.g. reload on a sub-route) is served via the `404.html` fallback and renders correctly
- [ ] 8.3 Confirm the CI job is green on a PR and that a forced failure uploads a usable Playwright trace
