## Context

The Guided activity feature spans three layers: the `ActivityCard` Blazor component (UI button + C# interop calls), `notification-helper.js` (IndexedDB state management and notification dispatch), and `service-worker.js` (Previous/Next action handling and notification cleanup). No other components or services reference guided functionality. The active-activity notification system in `notification-helper.js` is unrelated and must be preserved.

## Goals / Non-Goals

**Goals:**
- Eliminate all guided-specific code across the three affected files
- Leave the active-activity notification path (`startActiveNotification`, `updateActiveNotification`, `closeActiveNotification`) fully intact
- Leave the `Activity.Notes` field and its display in the card body untouched

**Non-Goals:**
- Removing the notification infrastructure shared with active activities (e.g. `requestPermission`, `_getRegistration`, `_getIconUrl`)
- Changing the notes editing or display experience
- Touching any service, model, or test file (no guided logic lives outside the three target files)

## Decisions

**Remove in-place rather than feature-flag**: The feature has no users or dependents; a flag would add complexity with no benefit. Hard delete is the right call.

**Retain shared notification helpers**: `requestPermission`, `_getRegistration`, `_getIconUrl` are used by the active-activity notification path. Only guided-exclusive members (`_storeName`, `_initDB`, `_storeState`, `_getState`, `_removeState`, `clearGuidedState`, `_splitNotes`, `startGuidedNotification`) are removed.

**No IndexedDB migration**: The `guidedNotifications` store was only written by the guided flow. Leaving stale data in existing clients is harmless; the store is never opened again after removal.

## Risks / Trade-offs

[Stale IndexedDB store in existing PWA installs] → No active reads or writes will occur after the JS is removed; existing data accumulates but causes no errors. Acceptable given the app's personal-use scale.

[Service worker cache] → PWA clients with a cached service worker will continue running the old SW until they reload after the new SW activates. The window is short and the old guided code causes no harm while the cache is being replaced.

## Migration Plan

1. Edit `ActivityCard.razor`: remove the Guided `<button>` block and the `HandleGuided` method; remove the `clearGuidedState` JS interop call from `ConfirmDelete`.
2. Edit `notification-helper.js`: delete guided-only members; keep the `window.notificationHelper` object and all active-activity methods.
3. Edit `service-worker.js`: remove `openGuidedNotificationsDb`, guided notification click handler branch, Previous/Next action handlers, and `notificationclose` guided cleanup.
4. Build and verify no compiler errors; smoke-test ActivityCard overlay (Edit, Duplicate, Finish, Delete still work).
