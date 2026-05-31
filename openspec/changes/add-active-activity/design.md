## Context

The app is a fully offline Blazor WASM PWA. All persistence goes through `IndexedDbStorageService` via JS interop. Currently, activities are created in a finished state with an optional user-entered duration. This change introduces an in-progress ("active") state so the app can track when an activity started and auto-compute the duration on finish.

Active state is intentionally transient: it represents activities the user is currently doing and should not be exported or restored across app versions.

## Goals / Non-Goals

**Goals:**
- Let users start an activity from the activity form and see it in a dedicated Active Activities section on Home
- Auto-populate duration (MMM:SS) when the user finishes an active activity
- Show real-time elapsed time on the Home page and as a notification on other pages
- Exclude active activity state from import/export

**Non-Goals:**
- Persisting active state across browser sessions (losing state on tab close is acceptable for v1)
- Supporting multiple simultaneous timers per activity type
- Background service-worker timing (Blazor timers run in the tab)

## Decisions

### Decision 1: Store active state in-memory only (no IndexedDB)

**Chosen**: `IActiveActivityService` holds active entries in a `Dictionary<int, DateTime>` (activityId → startTime) in memory only.

**Rationale**: Active activities are transient. Persisting them complicates import/export exclusion and adds IndexedDB schema migration risk. If the user closes the tab, the activity is simply no longer tracked — the partially-started activity remains in the normal activity store and the user can edit its duration manually.

**Alternative considered**: Store in IndexedDB in a separate object store. Rejected because it requires schema versioning, migration code, and explicit exclusion from the export serializer — complexity not warranted for transient state.

### Decision 2: Use a Blazor PeriodicTimer (or System.Threading.Timer) for elapsed display

**Chosen**: Each component that shows elapsed time subscribes to `IActiveActivityService.OnTimerTick` (an event raised every second by an internal `System.Threading.Timer`). Components call `StateHasChanged()` on tick.

**Rationale**: A single timer in the service avoids N timers for N active activities. Components simply listen while mounted and detach on dispose. Blazor's `InvokeAsync` ensures thread-safe UI dispatch.

**Alternative considered**: JS `setInterval` interop per component. Rejected because it requires round-trip JS calls for each update and adds lifecycle cleanup complexity.

### Decision 3: Duration format MMM:SS stored as a formatted string

**Chosen**: On finish, compute elapsed as `DateTime.UtcNow - startTime`, format as `{minutes:D3}:{seconds:D2}` (e.g., `002:45`), and write to `Activity.Duration` string field.

**Rationale**: The existing `Duration` field is a string. Reusing it avoids a model change and keeps the activity consistent with manually-entered durations. Formatting as MMM:SS matches the issue spec.

**Alternative considered**: Store elapsed seconds as an integer on the model. Rejected because it requires a model change and migration.

### Decision 4: Start/Stop toggle is an input-group addon on the Duration field

**Chosen**: The Duration input is wrapped in a Bootstrap input-group. A button appended at the right end shows a timer icon with a **Start** caption when the activity is not active, and a **Stop** caption when active. Clicking **Start** validates the form, saves the activity (reusing the existing save path), and registers it in `IActiveActivityService`. Clicking **Stop** computes elapsed duration, writes it to the Duration field, saves the activity, and unregisters it. The form remains open after both actions so the user can review or continue editing.

**Rationale**: Keeping the control inside the Duration field makes the relationship between the timer and the duration value immediately obvious. An input-group addon is a standard Bootstrap pattern that avoids an extra standalone button cluttering the form footer. The form stays open (no navigate-away) because the user may want to add notes or adjust the type before submitting with **Add**.

**Alternative considered**: A separate **Start** button alongside **Add** in the form footer. Rejected because it implies the same submission weight as **Add** and hides the connection to the duration value.

## Risks / Trade-offs

- **Timer drift on heavy tabs** → The 1-second timer is for display only; actual duration is always `DateTime.UtcNow - startTime` at finish time, so displayed time may lag by up to 1 s but the recorded duration will be exact.
- **Active state lost on tab close** → Documented non-goal. User is warned via the notification UI while activities are active.
- **Concurrent activities** → Multiple activities can be active simultaneously (the service stores a dictionary). The Home page section and notifications will show all of them.
- **Export exclusion** → The `IActiveActivityService` state is in-memory only, so export naturally excludes it — no extra exclusion logic needed in the export path.
