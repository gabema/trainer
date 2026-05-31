## Why

Users want to time activities as they happen, not just log them after the fact. A start/finish workflow lets the app automatically calculate duration, reducing manual entry and improving accuracy.

## What Changes

- Add a **Start/Stop** toggle button (timer icon + caption) appended to the end of the Duration input field on the activity form. Clicking **Start** saves the activity as active; while active, the button shows **Stop** and clicking it finishes the activity and auto-fills the Duration field with elapsed time.
- Introduce an **Active Activities** section on the Home page (between the "Activity by Goal Duration" graph and the main Activities list) that shows all currently running activities in real-time.
- Add a **Finish** button next to **Edit** for active activities; pressing it auto-fills the duration field with elapsed time (MMM:SS format) and removes the activity from the active list.
- Track active activity state as a separate list of activity IDs (not embedded in the Activity model).
- Exclude active activity tracking data from import/export operations.
- Show notification/alert messages on relevant pages when active activities exist, displaying their current elapsed durations updated in real-time.

## Capabilities

### New Capabilities
- `active-activities`: Start, monitor, and finish in-progress activities with automatic elapsed-time duration capture, a dedicated Home page section, and real-time duration notifications.

### Modified Capabilities
- `activity-filtering`: The Activities list needs to be aware of active-activity state so the Finish button can appear inline alongside Edit for active items.

## Impact

- **Models**: No changes to `Activity` record; new service/state to maintain a list of active activity IDs (likely stored in IndexedDB or browser local storage).
- **Services**: New `IActiveActivityService` (and implementation) to start, query, and finish active activities; timer logic for elapsed duration.
- **Pages/Components**: `Home.razor` gains an Active Activities section; the Duration input in `ActivityForm` gains a Start/Stop toggle button as an input-group addon; activity list rows gain a Finish button for active items.
- **Import/Export**: `ImportExportService` (or equivalent) must skip active-activity state when serializing/deserializing.
- **JS/interop**: Real-time countdown display will likely require a JS timer or Blazor timer to re-render elapsed time.
