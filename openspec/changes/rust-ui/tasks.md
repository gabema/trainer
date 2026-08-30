## 1. Framework scaffolding

- [x] 1.1 Add `dioxus` 0.7.10 to `trainer-web` with the `web` and `router` features only — no `fullstack`, `server`, `ssr`, `desktop` or `mobile`, since the app is a static bundle with no server. Pinned to stable rather than the 0.8 alpha. Kept under the existing `wasm32` target gate so `trainer-core`'s native tier is untouched
- [x] 1.2 Add `web-sys` features for geolocation, notifications, the intersection observer, and document access. `GetNotificationOptions` is **not** a valid feature in web-sys 0.3.104 — see task 3.4
- [x] 1.3 Add `Dioxus.toml` with `base_path`, `public_dir`, and `index_on_404`, and confirm a release build produces a `/trainer/`-rooted bundle with content-hashed assets. `wasm-opt` aborts on this toolchain and is left alone — see design
- [x] 1.4 Port `index.html`, dropping the Chart.js CDN tag, the Blazor runtime, its error UI, loading spinner and scoped-CSS bundle, and all six shim script tags; keeping the manifest, stylesheets, favicon, theme-color and service worker registration. Adds a static `<base href>`, which Dioxus does **not** emit — see design
- [x] 1.5 Reduce `theme.js` from 74 lines to a 9-line inline `<head>` script: read `prefers-color-scheme`, set `data-bs-theme`, listen for changes. The dead `window.themeManager` API, the unlistened `themechange` event, and the `chartHelper` call are gone
- [x] 1.6 Replace the `GenerateBuildInfo` MSBuild target with `option_env!("TRAINER_VERSION")`, defaulting to `dev`. Verified both ways: the footer renders `dev` in the browser, and a build with the variable set bakes the value into the binary

## 2. Shell, routing, and styling

- [x] 2.1 Port `MainLayout` and `TopNavBar`. `NavMenu` is **not** ported: 112 lines nothing renders, the Blazor template's default sidebar replaced by the tab bar and never deleted. `TopNavBar`'s `LocationChanged` subscription, `StateHasChanged` call and `IDisposable` are replaced by `Link`'s `active_class`
- [x] 2.2 Register all nine routes plus a NotFound catch-all. Verified in a browser: `/activity/5` renders with `id = 5`, and `/activity/abc` falls through to NotFound, matching Blazor's `{Id:int}` constraint
- [x] 2.3 Flatten `TopNavBar.razor.css` and `MainLayout.razor.css` into `app.css` — 145 live lines, not the 228 this task assumed, since 83 belonged to dead `NavMenu`. All 25 `!important` declarations and every `::deep` duplicate were dropped rather than translated: both were Blazor CSS-isolation artifacts. `main` is scoped to `.page main` so it does not reach further than Blazor's scoping allowed
- [x] 2.4 Port `AppVersionFooter` and place it on Home, Activities and Calendar — the three pages the spec names — rather than in `MainLayout`, which would also put it on the entry pages
- [x] 2.5 Verified in a browser: the shell renders, direct URLs and click navigation both work, the active tab updates without a reload, and the footer appears on the three pages the spec names

## 3. Reactive state

- [x] 3.1 Convert `OnChanged`, `OnTick` and `OnSlowTick` into three signals, with the 1s and 30s clocks running only while something is active — the behaviour `EnsureTimersRunning` / `StopTimers` had. Each operation is a stateless read-modify-write over the ported service, so localStorage stays authoritative for persistence and the signal for rendering, with nothing to drift between them
- [x] 3.2 Confirmed: neither component subscribes, unsubscribes, or calls a redraw. Three `IDisposable` implementations and six `InvokeAsync(StateHasChanged)` call sites are gone
- [x] 3.3 Port `ActiveActivities`. Verified in a browser against the spec scenarios: hidden with nothing active; showing name and `M:SS` elapsed when active; the clock advancing once a second and tracking real elapsed time within one tick; and Finish writing `durationSeconds`, clearing the entry, and removing the section
- [x] 3.4 Port `ActiveActivityNotification` against `web_sys::Notification` and `ServiceWorkerRegistration::show_notification_with_options`, preserving the `active-{id}` tag, the silent and non-renotifying options, and the base-path-aware icon URL. Note: web-sys 0.3.104 binds `get_notifications()` with **no filter argument**, so the shim's `getNotifications({tag})` becomes an unfiltered call plus a `Notification::tag()` comparison in Rust — behaviorally equivalent, and it keeps everything on typed bindings
- [x] 3.5 Request permission once on mount, short-circuiting when already granted or denied. Every notification call is best-effort and returns quietly with no permission or no ready service worker — verified by the app running cleanly in headless Chrome, which grants neither
- [x] 3.6 `notification-helper.js` is not carried into the Rust app — `public/` never included any of the shims. The copy under `Trainer/wwwroot/js/` is left in place deliberately: it is still referenced by the Blazor `index.html`, and deleting it now would break the shipping app for no gain when task 8.3 removes the whole directory

## 4. Browser APIs

- [x] 4.1 Implement geolocation against `web_sys::Geolocation`, preserving `enableHighAccuracy`, the 10s timeout, `maximumAge: 0`, and the `Denied` versus `Unavailable` distinction as an enum. The shim never rejected — it resolved with an error marker — and that shape is kept
- [x] 4.2 Implement `ScrollTrigger` against `web_sys::IntersectionObserver` with `threshold: 0.1`, viewport root and no margin. It owns its closure so the callback outlives the observer, and disconnects on drop — the shim's `dispose()`
- [x] 4.3 Preserve the issue #85 unobserve-then-observe workaround, covered by **two** tests: one asserting re-observing re-fires, and a contrast test driving a raw observer without the unobserve to assert it does not — so the workaround is shown necessary rather than merely present
- [x] 4.4 `infinite-scroll.js` is not carried into the Rust app; the remaining copy under `Trainer/wwwroot/js/` is still referenced by the Blazor build and goes with task 8.3, as with the other shims
- [x] 4.5 Add browser-tier coverage for all three: the observer's fire, re-arm, missing-element and drop-disconnects paths; geolocation resolving to a classified error rather than hanging; and notifications being a silent no-op without permission

## 5. Pages

- [x] 5.1 Port `Index` including recent activities and the goal summary
- [x] 5.2 Replace the Chart.js graph with declarative theme-aware SVG driven by CSS custom properties; delete `chart-helper.js`
- [x] 5.3 Port export and import, including the file picker and the download, without the `eval` and `alert` interop the Blazor version uses
- [x] 5.4 Port `ActivityEntry`, covering the `activity-duration`, `fractional-activity-amounts`, and `activity-location-capture` spec scenarios
- [x] 5.5 Port `DecimalAmountInput` using framework input binding; delete `decimal-input.js`
- [x] 5.6 Port `ActivityTypeEntry`, covering the `neutral-benefit` and `private-activity-types` spec scenarios
- [x] 5.7 Port `KnownLocationEntry`, covering the `known-locations` spec scenarios
- [x] 5.8 Port `ActivityCard` including the overlay actions, edit, finish, and delete
- [x] 5.9 Port `SearchFilter` and `Activities`, covering the `activity-filtering` spec scenarios including infinite scroll
- [x] 5.10 Port `Calendar` including the month view, search, and infinite scroll

> The Rust bundle carries no `js/` directory, so 5.2 and 5.5 are met by not
> porting `chart-helper.js` and `decimal-input.js`. The files themselves stay in
> `Trainer/wwwroot/js/` until 8.3, because `Index.razor` still calls
> `chartHelper.createGoalDurationChart` and `downloadFile` and the C# app has to
> keep running until then.

## 6. Service worker cutover

- [x] 6.1 Bump `CACHE_NAME` and remove the hardcoded `_framework` entries from `urlsToCache`
- [x] 6.2 Reduce install-time precaching to stable shell paths only, tolerating individual fetch failures without failing installation
- [x] 6.3 Add runtime cache-on-fetch for content-hashed assets, keyed to the current cache version
- [x] 6.4 Add `self.skipWaiting()` and `clients.claim()` for this release, with a comment explaining they are a one-time cutover measure to be removed
- [x] 6.5 Confirm activation deletes all non-current caches and does not touch IndexedDB or localStorage
- [x] 6.6 Verify `notificationclick` still focuses or opens the correct activity route
- [ ] 6.7 Verify against a profile with the previous PWA genuinely installed: old worker registered, old cache populated, real history present
- [ ] 6.8 Verify offline function after one online visit, including the chart
- [ ] 6.9 Verify deep links resolve both online and offline

> **6.7-6.9 are blocked on a browser, not on code.** Service worker
> registration fails for any script in the sandboxed browser available to this
> session (`An unknown error occurred when fetching the script`, reproduced with
> a one-line stub), and no Chrome instance is connected, so no live worker can
> be installed, activated, or taken offline here.
>
> What was verified instead, by loading the worker source and driving its
> `install` / `activate` / `fetch` handlers against real Cache Storage and real
> IndexedDB: install tolerates unreachable precache URLs; activate deletes
> `trainer-v2`, keeps `trainer-v3`, and leaves IndexedDB and localStorage
> untouched; `skipWaiting` and `clients.claim` are called; an offline deep link
> and a hosting 404 both fall back to the cached shell; a hashed asset is served
> from cache offline and is never re-fetched when present; a stable path serves
> stale and refreshes behind it; non-GET and cross-origin requests are passed
> through. Separately, the release build was served and every same-origin URL it
> requests was confirmed to fall under precache or the hashed-asset rule, with
> no cross-origin resources at all.
>
> A contrast run pins down why this matters: with `_framework/` removed — what
> the Rust deploy actually does — the previous worker caches **zero** entries,
> because `addAll` rejects as a batch and the `catch` swallows it. The app keeps
> working online, so the loss of offline mode leaves no trace.
>
> Still unproven, and only provable on a real deployment: that an installed
> Blazor PWA takes the new worker on next launch, that the app boots with the
> network genuinely off, and that a notification click navigates.

## 7. CI and deployment

- [x] 7.1 Remove the .NET setup, build, and test steps from `test.yml`
- [x] 7.2 Add the browser test tier coverage added in this change to `test.yml`
- [x] 7.3 Rewrite `deploy.yml`: Rust toolchain, wasm target, Dioxus CLI, `rust-cache`, `dx build --release`. It **must** be `dx build`, not `cargo build`: in release the router reads its base path from `DIOXUS_ASSET_ROOT`, a compile-time variable only the CLI sets, so a cargo-built release would resolve every route against the domain root
- [x] 7.4 Set `TRAINER_VERSION` from the release tag, stripping the leading `v`
- [x] 7.5 Remove the `sed` steps that rewrite `<base href>` in `index.html` and `404.html`. They go because the app now ships a static `<base href="/trainer/">` that is correct in dev too, **not** because Dioxus emits one — it does not
- [x] 7.6 Retain the `404.html` copy and confirm the manifest `start_url` is `/trainer/`
- [x] 7.7 Confirm a release build deploys to GitHub Pages and loads at the `/trainer/` subpath

> 7.2 needed no change: the browser tier already runs as one step, and the
> tests added by this change run under it.
>
> 7.7 was rehearsed rather than deployed. The workflow's own build and check
> commands were run locally against a `v1.4.2` tag, and the resulting bundle was
> served under `/trainer/` behind a server that answers unknown paths with
> `404.html` at status 404, the way GitHub Pages does. The app booted, the
> footer read `1.4.2`, every nav href was rooted at `/trainer/`, and
> `/trainer/activity/5` rendered the edit form rather than an error page. Only a
> real release tag can exercise Pages itself.
>
> Two things the rehearsal caught. `dx` minifies and re-serialises what it
> copies, so the bundled manifest is one line with its keys reordered — a check
> written against the source formatting would have failed every deploy. It also
> minifies `service-worker.js`, which means section 6 verified a file that is not
> the one that ships; the 18 checks were re-run against the bundled worker and
> all pass.

## 8. Retire the C# project

- [x] 8.1 Walk all nine capability specs plus `storage-data-compatibility` and confirm every scenario has a passing Rust test
- [x] 8.2 Confirm no ported test from `Trainer.Tests` was dropped rather than translated

> **8.2 found two real gaps, both now closed.** All 149 C# test methods were
> extracted and matched against the Rust tree. 66 are cited by name in a `Ports
> \`Name\`` doc comment; the other 83 were checked by behaviour, since the
> citation convention was applied unevenly. Every one of them has a counterpart
> except:
>
> - `LocalStorageServiceTests` (5 tests). `local.rs` had **no tests at all** —
>   the IndexedDB binding got eleven browser tests and the localStorage binding
>   got none, despite being where the active-activity set lives. Added seven
>   browser tests, including one that drives `ActiveActivityService` over the
>   real store across a simulated reload.
> - `DuplicateFrom_WithKnownLocationId_CopiesId` / `_CopiesNull`. The duplicate
>   path was inline in a component and untested. Extracted as
>   `models::duplicate_of` with a test. The two C# tests exist because the C#
>   listed every field by hand; `..source` makes that a compile-time guarantee,
>   so what the Rust test pins is what does *not* carry over — the id reset and
>   the new timestamp.
>
> Three C# tests are inapplicable by design rather than dropped:
> `FormatWhenDateTime_NoNowParameter_UsesCurrentTime` (the Rust signature always
> takes `now`, because the clock is injected),
> `ImportDataAsync_PreservesWallClockEvenWhenTheOffsetIsRecomputed` (Rust
> preserves the parsed offset instead of recomputing it — a documented
> divergence covered in `datetime.rs`), and the `RustInteropTests` cases, which
> tested the C# reading Rust output and have no meaning once no C# runs.
>
> **8.1.** The 164 scenarios across the ten specs split in two. Domain, storage
> and serialization scenarios have direct automated coverage — `storage-data-
> compatibility` in particular is covered scenario for scenario by `compat.rs`,
> `escaping.rs`, `datetime.rs`, `week.rs`, `migration.rs` and the golden
> fixtures. Rendering and interaction scenarios are satisfied by the
> implementation plus the manual verification recorded in sections 3-7, exactly
> as they were under the C#, whose suite was helpers and services only.
>
> Two scenarios lose their automated check with `Trainer.Tests`: "Data written
> by Rust is readable by the previous implementation" and "Rust export imports
> into C#". Both describe a direction that stops existing at this commit. The
> reverse direction — the one that still matters, since real user data was
> written by the C# — stays covered byte-for-byte by `compat.rs` against the
> committed `csharp-export.json`.
- [x] 8.3 Delete `Trainer/`, `Trainer.Tests/`, and `Trainer.sln`
- [x] 8.4 Remove `.NET`-specific entries from `.gitignore` and `.editorconfig` where no longer applicable
- [x] 8.5 Rewrite the tech-stack `context` block in `openspec/config.yaml`: stack, UI, storage, testing, project structure, and conventions all describe C# and Blazor
- [x] 8.6 Update `readme.md`, which describes .NET 10, Blazor, and the coverage claim throughout
- [x] 8.7 Update `CLAUDE.md` and `AGENTS.md` if they reference the C# layout
- [x] 8.8 Re-run GitNexus indexing so code intelligence reflects the Rust tree

> **8.3 broke the browser tier, which is how the deletion earned its keep.**
> `shim_interop_tests.rs` embeds `Trainer/wwwroot/js/indexeddb-storage.js` with
> `include_str!` and drives the real shim against the Rust storage layer, so
> deleting the C# tree stopped `trainer-web` compiling. The shim is now kept in
> `tests/fixtures/`: it is not C# source being retired but the program that
> wrote every existing user's IndexedDB, which makes it golden data on the same
> footing as `csharp-export.json`. Running it is the only way to check the
> format against the real thing rather than against a description of it.
>
> 8.4 was `.gitignore` only — 493 lines of `dotnet new gitignore` down to 54.
> `.editorconfig` had nothing .NET-specific in its six lines.
>
> 8.7 needed no manual edit: both files are generated by GitNexus and were
> rewritten by 8.8.
>
> 8.8 re-indexed 84 files out and 71 in, taking the graph from 868 nodes to
> 1,567. The index had been pinned to a June commit that predated the entire
> Rust tree, so every `impact` query against Rust returned "not found" and
> `detect_changes` reported "no changes" for the whole port. Both work now:
> `impact chart_series` resolves, and `detect_changes` correctly picks out the
> three `ActivityForm` flows touched by routing the duplicate path through
> `models::duplicate_of`.
