## Context

The app is a fully offline Blazor WASM PWA. All persistence goes through `IndexedDbStorageService` via JS interop. Currently, activities are created in a finished state with an optional user-entered duration. This change introduces an in-progress ("active") state so the app can track when an activity started and auto-compute the duration on finish.

Active state is session-like: it represents activities the user is currently doing, persists across page refreshes and is shared across tabs via `localStorage`, but is intentionally excluded from the app's import/export data format.

## Goals / Non-Goals

**Goals:**
- Let users start an activity from the activity form and see it in a dedicated Active Activities section on Home
- Auto-populate duration (MMM:SS) when the user finishes an active activity
- Show real-time elapsed time on the Home page and as a notification on other pages
- Exclude active activity state from import/export

**Non-Goals:**
- Real-time cross-tab sync (a second tab reads state on load, not live as the first tab changes it)
- Persisting active state across full browser restarts (localStorage is cleared when the user clears site data)
- Supporting multiple simultaneous timers per activity type
- Background service-worker timing (Blazor timers run in the tab)

## Decisions

### Decision 1: Mirror active state to localStorage (not IndexedDB)

**Chosen**: `IActiveActivityService` holds active entries in a `Dictionary<int, DateTime>` (activityId → `activity.When`) in memory as the primary source of truth, and mirrors every mutation to `localStorage` under the key `trainer_active_activities` as a JSON array (`[{"id":1,"startTime":"..."},...]`). On startup, `InitializeAsync()` reads the stored entries back into the in-memory dict. `IJSRuntime` is constructor-injected into the singleton service (valid in Blazor WASM where `IJSRuntime` is itself a singleton).

The `startTime` stored is `activity.When` (the activity's own recorded start time), not the wall-clock moment the Start button was pressed. Elapsed time therefore reflects how long the activity has been running since it was recorded as starting.

**Rationale**: `localStorage` requires no schema versioning or migration, is automatically excluded from IndexedDB export (the export path never touches `localStorage`), and enables state to survive page refreshes and be visible to new tabs/windows opened on the same origin. The in-memory dict remains the fast read path; `localStorage` writes are fire-and-forget so they never block the UI.

**Alternative considered**: Store in IndexedDB in a separate object store. Rejected because it requires schema versioning, migration code, and explicit exclusion from the export serialiser.

**Alternative considered**: Keep state in-memory only. Rejected because active state was lost on page refresh, which was unintuitive — a timed activity disappearing when the user navigates away is a poor experience.

### Decision 2: Use a Blazor PeriodicTimer (or System.Threading.Timer) for elapsed display

**Chosen**: Each component that shows elapsed time subscribes to `IActiveActivityService.OnTimerTick` (an event raised every second by an internal `System.Threading.Timer`). Components call `StateHasChanged()` on tick.

**Rationale**: A single timer in the service avoids N timers for N active activities. Components simply listen while mounted and detach on dispose. Blazor's `InvokeAsync` ensures thread-safe UI dispatch.

**Alternative considered**: JS `setInterval` interop per component. Rejected because it requires round-trip JS calls for each update and adds lifecycle cleanup complexity.

### Decision 3: Duration format MMM:SS stored as a formatted string

**Chosen**: On finish, compute elapsed as `DateTime.UtcNow - startTime`, format as `{minutes:D3}:{seconds:D2}` (e.g., `002:45`), and write to `Activity.Duration` string field.

**Rationale**: The existing `Duration` field is a string. Reusing it avoids a model change and keeps the activity consistent with manually-entered durations. Formatting as MMM:SS matches the issue spec.

**Alternative considered**: Store elapsed seconds as an integer on the model. Rejected because it requires a model change and migration.

### Decision 4: Start/Stop toggle is an input-group addon on the Duration field

**Chosen**: The Duration input is wrapped in a Bootstrap input-group. A button appended at the right end shows a timer icon with a **Start** caption when the activity is not active, and a **Stop** caption when active. Clicking **Start** validates the form, saves the activity (reusing the existing save path), and registers it in `IActiveActivityService` using `activity.When` as the start time. Clicking **Stop** computes elapsed duration as `DateTime.Now - activity.When`, writes it to the Duration field, saves the activity, and unregisters it. The form remains open after both actions so the user can review or continue editing.

**Rationale**: Keeping the control inside the Duration field makes the relationship between the timer and the duration value immediately obvious. An input-group addon is a standard Bootstrap pattern that avoids an extra standalone button cluttering the form footer. The form stays open (no navigate-away) because the user may want to add notes or adjust the type before submitting with **Add**.

**Alternative considered**: A separate **Start** button alongside **Add** in the form footer. Rejected because it implies the same submission weight as **Add** and hides the connection to the duration value.

### Decision 5: Browser (OS) notifications via the existing notification-helper.js

**Chosen**: Add three functions to the existing `notification-helper.js` — `startActiveNotification(activityId, name)`, `updateActiveNotification(activityId, name, elapsed)`, and `closeActiveNotification(activityId)`. Each notification uses a stable tag `active-{activityId}` so replacing it is silent (no new alert sound). The Blazor side calls these functions via `IJSRuntime` interop. Updates are sent every 30 seconds (not every second) to avoid excessive OS notification churn. There is no in-page banner component.

**Rationale**: `notification-helper.js` already has permission-request logic and service-worker notification wiring. Extending it keeps all notification code in one place. Using a tag to replace the notification silently means the user sees a live elapsed counter without being re-alerted every tick. 30-second update interval is a reasonable balance between freshness and noise.

**Alternative considered**: An in-page alert bar in `MainLayout.razor`. Rejected per user feedback — browser notifications work across browser tabs and at the OS level, which is more useful for a timing workflow.

**Where the notification interop is called**: A lightweight headless Blazor component (`ActiveActivityNotification.razor`) is mounted in `MainLayout.razor`. It renders no HTML, subscribes to `IActiveActivityService.OnChanged` and `OnSlowTick`, and calls the JS notification functions at the right moments. Note: `IJSRuntime` is also injected directly into `ActiveActivityService` for `localStorage` persistence (see Decision 1); this is valid in Blazor WASM where `IJSRuntime` is a singleton.

## Risks / Trade-offs

- **Timer drift on heavy tabs** → The 1-second timer is for display only; actual duration is always `DateTime.UtcNow - startTime` at finish time, so displayed time may lag by up to 1 s but the recorded duration will be exact.
- **Tab close before Finish** → If the user closes the tab mid-activity, the `localStorage` entry remains. Re-opening the app restores the activity with its original `When` as start time, so elapsed continues correctly. The OS notification stays visible until the user dismisses it.
- **No live cross-tab sync** → A second tab reads state on load via `InitializeAsync` but does not receive live updates when the first tab starts or finishes an activity. This is a documented non-goal; a `storage` event listener could add it in future.
- **Concurrent activities** → Multiple activities can be active simultaneously (the service stores a dictionary). The Home page section and notifications show all of them; all are persisted together in the single `localStorage` entry.
- **Export exclusion** → The export path serialises only IndexedDB data and never reads `localStorage`, so active state is naturally excluded with no extra code.
