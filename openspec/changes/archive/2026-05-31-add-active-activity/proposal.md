## Why

Users want to time activities as they happen, not just log them after the fact. A start/finish workflow lets the app automatically calculate duration, reducing manual entry and improving accuracy.

## What Changes

- Add a **Start/Stop** toggle button (timer icon + caption) appended to the end of the Duration input field on the activity form. Clicking **Start** saves the activity as active; while active, the button shows **Stop** and clicking it finishes the activity and auto-fills the Duration field with elapsed time.
- Introduce an **Active Activities** section on the Home page (between the "Activity by Goal Duration" graph and the main Activities list) that shows all currently running activities in real-time.
- Add a **Finish** button next to **Edit** for active activities; pressing it auto-fills the duration field with elapsed time (MMM:SS format) and removes the activity from the active list.
- Track active activity state as a separate list of activity IDs and start times persisted to `localStorage`, so state survives page refreshes and is visible across tabs and browser windows.
- Exclude active activity tracking data from import/export operations.
- Show browser (OS-level) notifications when activities are active, displaying their current elapsed durations. Notifications update periodically rather than in the page UI.

## Capabilities

### New Capabilities
- `active-activities`: Start, monitor, and finish in-progress activities with automatic elapsed-time duration capture, a dedicated Home page section, and real-time duration notifications.

### Modified Capabilities
- `activity-filtering`: The Activities list needs to be aware of active-activity state so the Finish button can appear inline alongside Edit for active items.

## Impact

- **Models**: No changes to `Activity` record; active state (activity ID → start time) is held in-memory and mirrored to `localStorage` under the key `trainer_active_activities`.
- **Services**: New `IActiveActivityService` (and implementation) to start, query, and finish active activities; timer logic for elapsed duration.
- **Pages/Components**: `Home.razor` gains an Active Activities section; the Duration input in `ActivityForm` gains a Start/Stop toggle button as an input-group addon; activity list rows gain a Finish button for active items. No in-page notification banner.
- **JS**: `notification-helper.js` gains `startActiveNotification`, `updateActiveNotification`, and `closeActiveNotification` functions for managing per-activity OS notifications.
- **Import/Export**: `ImportExportService` reads only IndexedDB; `localStorage` is never touched by the export path, so active state is excluded naturally.
