## 1. Framework scaffolding

- [ ] 1.1 Add `dioxus` and `dioxus-router` to `trainer-rs`, configured for client-side rendering only
- [ ] 1.2 Add `web-sys` features for `Geolocation`, `Notification`, `ServiceWorkerRegistration`, and `IntersectionObserver`
- [ ] 1.3 Configure the Dioxus CLI with base-path support and confirm a release build produces a working `/trainer/`-rooted bundle locally
- [ ] 1.4 Port `index.html`: drop the Chart.js CDN tag, the Blazor script, and the Blazor error UI; keep the manifest, stylesheets, and service worker registration
- [ ] 1.5 Reduce `theme.js` to a minimal inline `<head>` script — read `prefers-color-scheme`, set `data-bs-theme`, listen for changes; delete `window.themeManager`, the `themechange` event, and the `chartHelper` call
- [ ] 1.6 Replace the `GenerateBuildInfo` MSBuild target with `option_env!("TRAINER_VERSION")`, defaulting to `dev`

## 2. Shell, routing, and styling

- [ ] 2.1 Port `MainLayout`, `NavMenu`, and `TopNavBar`
- [ ] 2.2 Register all nine routes, including typed integer parameters on `/activity/{id}`, `/activity-type/{id}`, and `/known-location/{id}`
- [ ] 2.3 Flatten `TopNavBar.razor.css`, `NavMenu.razor.css`, and `MainLayout.razor.css` into `app.css` under a component-prefix convention
- [ ] 2.4 Port `AppVersionFooter`, satisfying the `app-version-footer` spec scenarios
- [ ] 2.5 Confirm the app builds, routes, and renders an empty shell before porting any page

## 3. Reactive state

- [ ] 3.1 Convert `ActiveActivityService`'s `OnChanged`, `OnTick`, and `OnSlowTick` events to signals, retaining the 1s and 30s timers
- [ ] 3.2 Confirm no component needs explicit subscribe, unsubscribe, or manual redraw
- [ ] 3.3 Port `ActiveActivities`, covering the `active-activities` spec scenarios
- [ ] 3.4 Port `ActiveActivityNotification` against `web_sys::Notification` and `ServiceWorkerRegistration::show_notification_with_options`, preserving the `active-{id}` tag, the silent and non-renotifying options, and the base-path-aware icon URL
- [ ] 3.5 Implement notification permission request and the silent no-op when permission is absent
- [ ] 3.6 Delete `notification-helper.js`

## 4. Browser APIs

- [ ] 4.1 Implement geolocation against `web_sys::Geolocation`, preserving `enableHighAccuracy`, the 10s timeout, and the `denied` versus `unavailable` error distinction; delete `geolocation-helper.js`
- [ ] 4.2 Implement the intersection observer against `web_sys::IntersectionObserver` with `threshold: 0.1`
- [ ] 4.3 Preserve the issue #85 workaround: unobserve before observing so the callback re-fires when the trigger stays within the viewport, and cover it with a test
- [ ] 4.4 Delete `infinite-scroll.js`
- [ ] 4.5 Add `wasm-bindgen-test` coverage for geolocation, notifications, and the observer

## 5. Pages

- [ ] 5.1 Port `Index` including recent activities and the goal summary
- [ ] 5.2 Replace the Chart.js graph with declarative theme-aware SVG driven by CSS custom properties; delete `chart-helper.js`
- [ ] 5.3 Port export and import, including the file picker and the download, without the `eval` and `alert` interop the Blazor version uses
- [ ] 5.4 Port `ActivityEntry`, covering the `activity-duration`, `fractional-activity-amounts`, and `activity-location-capture` spec scenarios
- [ ] 5.5 Port `DecimalAmountInput` using framework input binding; delete `decimal-input.js`
- [ ] 5.6 Port `ActivityTypeEntry`, covering the `neutral-benefit` and `private-activity-types` spec scenarios
- [ ] 5.7 Port `KnownLocationEntry`, covering the `known-locations` spec scenarios
- [ ] 5.8 Port `ActivityCard` including the overlay actions, edit, finish, and delete
- [ ] 5.9 Port `SearchFilter` and `Activities`, covering the `activity-filtering` spec scenarios including infinite scroll
- [ ] 5.10 Port `Calendar` including the month view, search, and infinite scroll

## 6. Service worker cutover

- [ ] 6.1 Bump `CACHE_NAME` and remove the hardcoded `_framework` entries from `urlsToCache`
- [ ] 6.2 Reduce install-time precaching to stable shell paths only, tolerating individual fetch failures without failing installation
- [ ] 6.3 Add runtime cache-on-fetch for content-hashed assets, keyed to the current cache version
- [ ] 6.4 Add `self.skipWaiting()` and `clients.claim()` for this release, with a comment explaining they are a one-time cutover measure to be removed
- [ ] 6.5 Confirm activation deletes all non-current caches and does not touch IndexedDB or localStorage
- [ ] 6.6 Verify `notificationclick` still focuses or opens the correct activity route
- [ ] 6.7 Verify against a profile with the previous PWA genuinely installed: old worker registered, old cache populated, real history present
- [ ] 6.8 Verify offline function after one online visit, including the chart
- [ ] 6.9 Verify deep links resolve both online and offline

## 7. CI and deployment

- [ ] 7.1 Remove the .NET setup, build, and test steps from `test.yml`
- [ ] 7.2 Add the browser test tier coverage added in this change to `test.yml`
- [ ] 7.3 Rewrite `deploy.yml`: Rust toolchain, wasm target, Dioxus CLI, `rust-cache`, `dx build --release`
- [ ] 7.4 Set `TRAINER_VERSION` from the release tag, stripping the leading `v`
- [ ] 7.5 Remove the `sed` steps that rewrite `<base href>` in `index.html` and `404.html`
- [ ] 7.6 Retain the `404.html` copy and confirm the manifest `start_url` is `/trainer/`
- [ ] 7.7 Confirm a release build deploys to GitHub Pages and loads at the `/trainer/` subpath

## 8. Retire the C# project

- [ ] 8.1 Walk all nine capability specs plus `storage-data-compatibility` and confirm every scenario has a passing Rust test
- [ ] 8.2 Confirm no ported test from `Trainer.Tests` was dropped rather than translated
- [ ] 8.3 Delete `Trainer/`, `Trainer.Tests/`, and `Trainer.sln`
- [ ] 8.4 Remove `.NET`-specific entries from `.gitignore` and `.editorconfig` where no longer applicable
- [ ] 8.5 Rewrite the tech-stack `context` block in `openspec/config.yaml`: stack, UI, storage, testing, project structure, and conventions all describe C# and Blazor
- [ ] 8.6 Update `readme.md`, which describes .NET 10, Blazor, and the coverage claim throughout
- [ ] 8.7 Update `CLAUDE.md` and `AGENTS.md` if they reference the C# layout
- [ ] 8.8 Re-run GitNexus indexing so code intelligence reflects the Rust tree
