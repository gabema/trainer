## Context

`rust-foundation` leaves the repository with a proven Rust crate containing domain, services, and storage, and a Blazor app still shipping on top of the old stack. This change swaps the view layer and retires the C# project.

The UI is ~2,200 lines of `.razor` across six pages and five components, nine routes, and 874 lines of JavaScript interop. Unlike the foundation work, almost none of this translates mechanically — `.razor` markup with two-way binding has no direct Rust equivalent, so this is the change where the effort actually lands.

The framework choice was made against a specific criterion: the motivation for this rewrite is language preference, not bundle size or performance. That removes the argument for the smallest possible payload and promotes the question of what it feels like to port 2,200 lines of markup.

## Goals / Non-Goals

**Goals:**
- Port every page, component, and route with behavior preserved, verified against the nine existing capability specs.
- Reduce the JavaScript interop layer to what genuinely cannot be Rust.
- Migrate installed PWAs cleanly, without stranding users or serving a half-updated shell.
- Retire `Trainer/`, `Trainer.Tests/`, and the .NET half of CI.

**Non-Goals:**
- Redesigning the UI. This is a port; visual and interaction changes belong in separate changes.
- Optimizing bundle size.
- Replacing Bootstrap.
- Correcting the week-key and timestamp-format quirks documented in `rust-foundation`.

## Decisions

### Dioxus over Leptos and Yew

| | Dioxus | Leptos | Yew |
|---|---|---|---|
| Porting 2,200 lines of markup | RSX hot reload | rebuild loop | rebuild loop |
| Learning curve alongside Rust | hooks forgive mistakes | signal and closure lifetimes surface as errors inside macro output | explicit but boilerplate-heavy |
| Signals available | yes | yes | no |
| Momentum | active | active | mature, slowing |

With size off the table, the dominant cost is hand-porting markup, and RSX hot reload is the largest single lever on how that goes. Leptos is the more elegant model and would win if bytes mattered; they do not. Yew is closest to Blazor's mental model, which is a reason to reject it — the point is to be writing Rust, not Blazor in Rust.

An earlier draft of this analysis favored Leptos partly on the belief that signals were Leptos-only. That is outdated: Dioxus has had signals since 0.5 and made them primary in 0.6, so the argument does not distinguish the two.

### `ActiveActivityService`'s event plumbing becomes signals

The existing service is a hand-rolled fine-grained reactive system:

```csharp
public event Action? OnChanged;   // start / finish
public event Action? OnTick;      //  1s → elapsed clocks
public event Action? OnSlowTick;  // 30s → notification refresh
```

Three components subscribe in `OnInitialized`, unsubscribe in `Dispose`, and wrap every callback in `InvokeAsync(StateHasChanged)`. Under signals the events, the `IDisposable` implementations, and the manual redraw calls all disappear; the timers remain, writing to signals.

### The chart becomes declarative SVG, not a canvas library

The Index chart shows activity counts by type. Under a view framework that is a `<svg>` with a loop emitting `<rect>` elements — roughly 60 lines of markup replacing 353 lines of `chart-helper.js` plus a CDN dependency.

This also fixes a real defect. Chart.js is loaded from jsdelivr at `index.html:33` and never appears in the service worker's precache list, so on a cold cache with no network the chart does not render — in an app whose stated promise is full offline function.

*Alternative considered:* a Rust charting crate. Rejected — one bar chart does not justify a dependency, and the existing `chart-helper.js` already carries theme-awareness logic that is simpler to express as CSS custom properties in inline SVG.

### Interop is reduced to what physics requires

| File | Fate |
|---|---|
| `chart-helper.js` 353 | deleted — declarative SVG |
| `indexeddb-storage.js` 208 | deleted in `rust-foundation` |
| `notification-helper.js` 99 | deleted — `web_sys::Notification`, `ServiceWorkerRegistration::show_notification_with_options`, `get_notifications` |
| `theme.js` 74 | reduced to ~6 inline lines |
| `decimal-input.js` 60 | deleted — framework input binding |
| `infinite-scroll.js` 53 | deleted — `web_sys::IntersectionObserver` |
| `geolocation-helper.js` 27 | deleted — `web_sys::Geolocation` |
| `service-worker.js` 86 | rewritten, stays JavaScript |

Two cannot move. **A service worker runs in its own worker context registered from a JavaScript URL** — WASM cannot be one, and wrapping an 86-line cache-first shell in WASM would be absurd. **Theme must be applied before first paint**; WASM loads asynchronously afterward, so applying it from Rust means a flash of the wrong theme on every cold start.

The theme script still shrinks by 68 lines. What it currently contains beyond reading `prefers-color-scheme` and setting `data-bs-theme` is a `window.themeManager` API annotated "for potential Blazor interop" that no C# code calls, a `themechange` `CustomEvent` nothing listens for, and a `chartHelper.updateAllCharts` call that dies with Chart.js.

*Alternative considered:* eliminating the script entirely by defining the dark palette under `@media (prefers-color-scheme: dark)` instead of `[data-bs-theme="dark"]`. Rejected — it means overriding vendored Bootstrap selectors in `app.css` to save six lines.

### Service worker: runtime caching, not a generated manifest

The current worker precaches a hardcoded list including `_framework/blazor.webassembly.js` and `_framework/wasm/dotnet.wasm`. Both cease to exist, and the replacement is worse: the Rust build emits content-hashed filenames that change every release. Since `cache.addAll` rejects its entire install if any URL 404s, a stale list means the worker silently fails to install and offline mode dies while the app still appears to work online.

Precache only the stable shell — `/`, `index.html`, `manifest.json`, `favicon.png`, the stylesheets — and let hashed assets populate through runtime cache-on-fetch.

*Alternative considered:* a CI step that globs the build output and injects the asset list. Rejected — it adds build-time codegen and couples the worker to the workflow, where runtime caching needs neither.

### One-time forced takeover, then remove it

The existing worker is cache-first for everything, so an installed user can keep being served a cached Blazor `index.html` until the browser re-fetches `service-worker.js` and every old client closes. Bumping `CACHE_NAME` purges stale entries on activate but does not force the handoff.

This release therefore ships `self.skipWaiting()` plus `clients.claim()`. Both should be removed in a later change: permanently skipping the waiting phase is how a user ends up with a document and assets from different builds, which is exactly the failure this change exists to prevent. They are correct for one cutover and wrong as a standing policy.

### Scoped CSS is flattened

`TopNavBar.razor.css`, `NavMenu.razor.css`, and `MainLayout.razor.css` rely on Blazor's CSS isolation and its generated `Trainer.styles.css`. No Rust framework provides an equivalent. The 228 lines fold into `app.css` under a component-prefix convention. A scoping crate would preserve the hygiene at the cost of a dependency for three files.

### Version injection collapses to an environment variable

The `GenerateBuildInfo` MSBuild target writes a `BuildInfo.g.cs` at build time for a single line in `AppVersionFooter.razor`. In Rust this is `option_env!("TRAINER_VERSION").unwrap_or("dev")`, with the workflow setting the variable from the release tag. The `app-version-footer` capability is unchanged; roughly 90% of the machinery goes away.

### Dioxus does not emit a `<base>` tag — the app supplies one

Found while wiring the CLI, and it corrects this document's earlier claim that
"the `sed` steps die, the CLI has first-class base-path support". Half right.

`[web.app] base_path` rewrites the URLs Dioxus *itself* injects — its wasm and
JS bundle — and in dev builds it emits a `<meta name="…asset root…">` for the
router. It does **not** emit a `<base>` element, so the stylesheet, favicon and
manifest links in a custom `index.html` stay relative and resolve against the
current route. On a deep link such as `/trainer/activity/5` they would resolve
to `/trainer/activity/css/app.css` and 404.

The app therefore ships a static `<base href="/trainer/" />`. That is correct in
both environments because `dx serve` nests the dev server under `base_path`
too — `router.nest_service(&base_path, …)` — so local development is also at
`/trainer/`, unlike Blazor where local was `/` and deploy rewrote it.

Net effect on task 7.5: the `<base href>` `sed` steps still go, but because the
value is now identical everywhere rather than because the CLI handles it.

Verified in a browser: every asset resolves under `/trainer/`, and the only 404
is `GET /`, which is correct since the app is not served from the domain root.

### `wasm-opt` aborts on this toolchain

`dx build --release` runs `wasm-opt` by default and it exits with SIGABRT:

```
compile unit size was incorrect (this may be an unsupported version of DWARF)
```

`dx` reports the build as successful regardless, so the wasm ships unoptimized
while the log says everything passed. Neither `[profile.release] debug = false`
nor removing the `[web.wasm_opt]` tuning avoids it; `strip` is worse, since dx's
strip step cannot load `libLLVM.dylib` here.

Left as is rather than worked around. Bundle size is explicitly not a goal of
this rewrite, the emitted bundle is correct, and the failure is a local
binaryen/DWARF incompatibility rather than anything this change introduced.
Whether it also occurs on the Linux CI runner is checked at task 7.7.

### `NavMenu` is dead code and is not ported

`MainLayout` renders `TopNavBar`. `NavMenu` appears nowhere else in the C#
source — not in a `.razor`, not in a `.cs`. It is the Blazor project template's
default sidebar nav, replaced by the tab bar and never deleted.

That is 112 lines nothing renders: `NavMenu.razor` (29) plus
`NavMenu.razor.css` (83). Porting it would mean translating markup no user has
ever seen. Task 2.1 named it; task 2.3 counted its stylesheet in the 228 lines
to flatten. The real figure is 145 lines across two files.

The third piece of dead code this rewrite has surfaced, after
`EmptyStringAsNullConverter` and the absent `WeekHelperTests`.

### Flattening dropped every `!important`

The scoped stylesheets carried 25 `!important` declarations and a set of
`::deep` selector duplicates. Both were artifacts of Blazor's CSS isolation:
`::deep` let a scoped sheet reach into a child component's markup, and the
`!important` flags won the specificity fights that scoping created — `NavLink`
rendered the anchor, so `.nav-tab` and `::deep a.nav-tab` competed.

Once the rules are global, the plain selector already matches and nothing
competes. All 25 `!important` declarations and every `::deep` duplicate were
removed rather than translated.

Verified in a browser rather than assumed: the tab bar renders as a
three-column grid with flex tabs, and the active tab resolves to weight 600,
the dark-mode background and the link-coloured underline, matching the Blazor
design exactly with no `!important` anywhere.

### The router replaces the active-tab machinery

`TopNavBar.razor` computed the active tab by hand — reading
`NavigationManager.Uri`, stripping the base URI, comparing prefixes — then
subscribed to `LocationChanged`, called `StateHasChanged`, and implemented
`IDisposable` purely to re-render on navigation.

`Link`'s `active_class` does all of it declaratively. The subscription, the
manual redraw and the disposal are gone, and the active class was observed
updating on a click without a reload.

### The release build must go through `dx`

`dioxus-router` resolves the base path differently per profile: in debug it
reads the `<meta>` element the CLI injects, and in release it reads
`option_env!("DIOXUS_ASSET_ROOT")` — a **compile-time** variable the CLI sets
during `dx build`.

A release built with plain `cargo build` would therefore have no base path and
every route would resolve against the domain root. Task 7.3 must build with
`dx`, not `cargo`.

### `serviceWorker.ready` never settles without a worker

Found by the notification test hanging rather than failing.

`navigator.serviceWorker.ready` does not reject when no service worker is
registered — it simply never resolves. `notification-helper.js` awaited it
inside a `try`/`catch`, which cannot catch a promise that never settles, so on
a page without a registered worker every notification call hangs forever. In
the C# that was invisible: the calls were fire-and-forget, so a hung task just
sat there and no notification appeared.

The port races the wait against a two-second timeout, so the same situation
becomes a clean "no notification" — which every caller already handles, since
the whole path is best-effort by design. The activity timer must keep working
whether or not notifications do.

This is a fix, not a port, and it is deliberate: reproducing a hang faithfully
would be fidelity to a defect with no upside.

### The issue #85 workaround is load-bearing, and now proven so

`observe()` on an already-observed element is a no-op, so when a sparse filtered
list leaves the trigger inside the viewport, nothing new intersects and loading
stalls. `ScrollTrigger::observe` unobserves first to force a fresh evaluation.

Two tests cover it rather than one. The first asserts that re-observing
re-fires. The second drives a **raw** `IntersectionObserver` the same way
without the unobserve and asserts it does *not* fire again — so the workaround
is shown to be necessary rather than merely present, and if a browser ever
changes that behaviour the contrast test says so directly.

### Deploy workflow

| Step today | After |
|---|---|
| `setup-dotnet`, `dotnet restore` | Rust toolchain, wasm target, Dioxus CLI, `rust-cache` |
| `dotnet build -p:InformationalVersion=` | `TRAINER_VERSION` environment variable |
| `dotnet publish -p:BasePath=/trainer/` | `dx build --release` with base path from config |
| `sed` `<base href>` in index.html and 404.html | removed — first-class base-path support |
| `cp index.html 404.html` | retained — GitHub Pages has no SPA fallback |
| `sed` manifest `start_url` | retained, or committed as `/trainer/` |
| Pages setup, upload, deploy | unchanged |

## Risks / Trade-offs

**Installed users stranded on the Blazor shell** → `skipWaiting` plus `clients.claim` for this release, cache version bumped, and the scenarios in `pwa-update-migration` verified against a profile that actually has the old worker installed. Testing this on a clean profile proves nothing.

**Half-updated shell served during cutover** → Runtime caching keyed to a single cache version, so a document and its assets cannot come from different builds.

**Offline mode fails silently** → The failure is invisible online, which is how it would reach production. Explicit offline verification is a task, not an assumption.

**Newly written `web-sys` code replaces proven shims** → Geolocation, notifications, and the intersection observer move to the browser test tier. Accepted: it is the direct cost of the interop reduction the change is for.

**The issue #85 observer workaround is easy to lose** → `infinite-scroll.js` unobserves before observing, because re-observing an already-observed target is a no-op and the callback would never re-fire when the trigger stays in the viewport with sparse filtered results. Carried over with a test.

**Long-lived branch with nothing verifiable end-to-end** → Partly mitigated by the two-change split, but this change is still large. Order pages so a runnable app exists early: shell and routing, then Index, then the rest.

## Migration Plan

1. Ship with the cache version bumped and forced takeover enabled.
2. Verify against a profile with the previous PWA genuinely installed — old worker registered, old cache populated, real activity history present.
3. Confirm history, activity types, known locations, and in-progress activities survive.
4. Confirm offline function after one online visit.

**Rollback:** re-deploy the last Blazor release. Because this change forces takeover, a rollback also needs its own cache version bump, or clients hold the Rust shell. Worth rehearsing before the cutover rather than discovering during it.

## Open Questions

- Does Dioxus's base-path support cover the manifest `start_url` and the 404 copy, or do those `sed` steps stay?
- Should `skipWaiting` removal be a task in a follow-up change, or a `TODO` with an issue?
- Do the six pages port in one change, or is Calendar (437 lines, month view plus search plus infinite scroll) worth splitting out?
- Does `readme.md` get rewritten here or separately? It describes .NET 10, Blazor, and >90% unit test coverage throughout.
