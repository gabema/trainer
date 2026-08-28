## Why

`rust-foundation` ports the domain, services, and storage to Rust and proves data compatibility, but leaves the Blazor app shipping. This change replaces the view layer, retires the C# project, and cuts the deployed PWA over to the Rust build.

The cutover carries a risk the foundation change does not: the app is installable and offline-first, so existing users are running a service worker that caches the Blazor shell **cache-first**. A naive deploy can leave installed users on the old app indefinitely, or — worse — in a half-updated state where a fresh `index.html` is paired with stale cached assets. The service worker migration is treated here as first-class behavior with its own spec, not as a deployment footnote.

## What Changes

- Add Dioxus and port all six pages and five components from `.razor` to Rust: Index, Activities, Calendar, ActivityEntry, ActivityTypeEntry, KnownLocationEntry, ActivityCard, ActiveActivities, ActiveActivityNotification, DecimalAmountInput, SearchFilter.
- Port the nine routes, including the typed integer parameters on `/activity/{id}`, `/activity-type/{id}`, and `/known-location/{id}`.
- Replace the hand-rolled reactive layer in `ActiveActivityService` — three `event Action` members, three `IDisposable` unsubscribes, and manual `StateHasChanged` calls across three components — with signals.
- **Reduce the JavaScript interop layer from 874 lines to roughly 6.** Delete `chart-helper.js` (353), `indexeddb-storage.js` (208), `notification-helper.js` (99), `decimal-input.js` (60), `infinite-scroll.js` (53), and `geolocation-helper.js` (27). Reduce `theme.js` (74) to a small inline `<head>` script.
- **BREAKING (build/deploy only):** Remove the Chart.js CDN dependency. The activity chart becomes declarative SVG in the view. This also fixes a live defect: Chart.js is loaded from jsdelivr and never cached by the service worker, so the chart does not render offline on a cold cache.
- Rewrite `service-worker.js` caching so it does not hardcode framework asset paths, and add a one-time forced takeover so installed PWAs migrate off the Blazor shell.
- Flatten the 228 lines of Blazor-scoped CSS (`TopNavBar.razor.css`, `NavMenu.razor.css`, `MainLayout.razor.css`) into `app.css` with a naming convention, since no Rust framework provides Blazor's CSS isolation.
- Replace the 20-line `GenerateBuildInfo` MSBuild target with a build-time environment variable read via `option_env!`.
- Rewrite `deploy.yml` for the Rust toolchain, and remove the .NET half of `test.yml`.
- **BREAKING:** Delete `Trainer/`, `Trainer.Tests/`, and `Trainer.sln`.
- Rewrite the tech-stack `context` block in `openspec/config.yaml`, which describes C#, Blazor, xUnit, and Chart.js throughout.

## Capabilities

### New Capabilities

- `pwa-update-migration`: Governs how an already-installed Blazor PWA transitions to the Rust build — that the update is actually taken rather than deferred indefinitely, that stale cached assets are purged, that no half-updated state is served, and that offline capability survives the cutover under content-hashed filenames.

### Modified Capabilities

None. The nine existing capability specs describe behavior this change preserves. `app-version-footer` in particular continues to be satisfied, by a different mechanism.

## Impact

**Added**
- `dioxus`, `dioxus-router`; `web-sys` features for `Geolocation`, `Notification`, `ServiceWorkerRegistration`, `IntersectionObserver`.
- Dioxus CLI in both workflows.

**Removed**
- `Trainer/`, `Trainer.Tests/`, `Trainer.sln`, and the .NET toolchain from CI.
- Six of seven JavaScript interop files.
- The Chart.js CDN script tag.
- The `sed` steps in `deploy.yml` that rewrite `<base href>`, replaced by the Dioxus CLI's base-path configuration.

**Retained**
- `service-worker.js` — rewritten, but it stays JavaScript. A service worker runs in its own worker context registered from a JS URL; WASM cannot be one.
- ~6 inline lines of theme script in `<head>`. WASM loads after first paint, so a theme applied from Rust would flash the wrong theme on every cold start. 68 of the current 74 lines are removable regardless: a dead `window.themeManager` API that no C# code calls, a `themechange` event nothing listens for, and a `chartHelper` coupling that dies with Chart.js.
- `cp index.html 404.html` in the deploy workflow — GitHub Pages has no SPA fallback, so deep links such as `/trainer/activity/5` 404 without it. Framework-independent.
- Bootstrap 5 CSS and `app.css`.

**Risks**
- Installed users stranded on the old service worker, or served a half-updated shell.
- Content-hashed asset filenames break the current hardcoded `urlsToCache`; `cache.addAll` rejects its entire install if any URL 404s, which would disable offline mode silently while the app still works online.
- The browser test tier grows to cover geolocation, notifications, and the intersection observer, all newly written against `web-sys` rather than proven JS shims.
- The `unobserve`-then-`observe` workaround in `infinite-scroll.js` (issue #85) is subtle and easy to lose in translation.
