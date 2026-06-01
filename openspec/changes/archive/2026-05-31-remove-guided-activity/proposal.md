## Why

The Guided activity feature — which parses an activity's notes into lines and steps through them via push notifications — adds significant complexity across the UI, JavaScript, and service worker layers without delivering meaningful value to users. Removing it simplifies the codebase and eliminates the only feature dependent on notification permission prompts and IndexedDB-backed notification state.

## What Changes

- **BREAKING** Remove the "Guided" button from the `ActivityCard` overlay and delete the `HandleGuided()` C# method
- Remove the `clearGuidedState` JS call from `ConfirmDelete()` in `ActivityCard`
- Delete all guided-specific code from `notification-helper.js`: `_storeName`, `_initDB`, `_storeState`, `_getState`, `_removeState`, `clearGuidedState`, `_splitNotes`, and `startGuidedNotification`
- Remove guided notification handling (Previous/Next actions, IndexedDB state, notification click routing) from `service-worker.js`

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

_(none — no spec-level requirements are changing; this is a code-only removal of an unspecced feature)_

## Impact

- `Trainer/Components/ActivityCard.razor` — remove guided button, `HandleGuided` method, and `clearGuidedState` JS interop call in delete flow
- `Trainer/wwwroot/js/notification-helper.js` — remove guided-only members; retain `requestPermission`, `_getRegistration`, `_getIconUrl`, `startActiveNotification`, `updateActiveNotification`, `closeActiveNotification`
- `Trainer/wwwroot/service-worker.js` — remove guided IndexedDB open helper, guided notification click handler, Previous/Next action handling, and `notificationclose` guided cleanup
