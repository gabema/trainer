## 1. ActivityCard Component

- [x] 1.1 Remove the Guided `<button>` block (and its conditional wrapper) from the overlay in `ActivityCard.razor`
- [x] 1.2 Delete the `HandleGuided()` method from the `@code` block
- [x] 1.3 Remove the `clearGuidedState` JS interop call (and surrounding try/catch) from `ConfirmDelete()`

## 2. Notification Helper JS

- [x] 2.1 Delete `_storeName`, `_initDB`, `_storeState`, `_getState`, `_removeState` from `notification-helper.js`
- [x] 2.2 Delete `clearGuidedState` and `_splitNotes` from `notification-helper.js`
- [x] 2.3 Delete `startGuidedNotification` from `notification-helper.js`

## 3. Service Worker

- [x] 3.1 Delete `openGuidedNotificationsDb` helper function from `service-worker.js`
- [x] 3.2 Remove the guided notification click handler branch (tag starts with `guided-`) from the `notificationclick` event listener
- [x] 3.3 Remove the Previous/Next action handlers for guided notifications from `service-worker.js`
- [x] 3.4 Remove the `notificationclose` guided cleanup block from `service-worker.js`

## 4. Verification

- [x] 4.1 Build the project (`dotnet build`) with no errors or warnings related to removed code
- [x] 4.2 Smoke-test the ActivityCard overlay: Edit, Duplicate, Finish, and Delete actions still work correctly
- [x] 4.3 Confirm no JS console errors on activity card interaction
