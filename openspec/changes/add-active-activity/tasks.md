## 1. Active Activity Service

- [ ] 1.1 Create `IActiveActivityService` interface with `Start(int activityId)`, `Finish(int activityId)`, `IsActive(int activityId)`, `GetAll()` returning `IReadOnlyDictionary<int, DateTime>`, and an `OnChanged` event
- [ ] 1.2 Implement `ActiveActivityService` backed by an in-memory `Dictionary<int, DateTime>` (activityId → UTC start time); raise `OnChanged` on Start/Finish
- [ ] 1.3 Add a 1-second `System.Threading.Timer` in `ActiveActivityService` that raises an `OnTick` event; stop the timer when the dictionary is empty
- [ ] 1.4 Register `ActiveActivityService` as a singleton in `Program.cs`
- [ ] 1.5 Write `ActiveActivityServiceTests` covering Start, Finish, IsActive, and GetAll

## 2. Duration Formatting Helper

- [ ] 2.1 Add a `FormatElapsed(TimeSpan elapsed)` method to `DateTimeHelper` (or a new `DurationHelper`) that returns `MMM:SS` format (e.g., `002:45`)
- [ ] 2.2 Write unit tests for `FormatElapsed` covering zero, single-digit seconds, and values >= 999 minutes

## 3. Activity Form — Duration Start/Stop Button

- [ ] 3.1 Wrap the Duration input in a Bootstrap input-group and append a toggle button at the right end showing a timer icon and **Start** caption
- [ ] 3.2 Wire the **Start** click: validate the form, save the activity (reusing the existing save path), call `IActiveActivityService.Start(activityId)`, and update the button to show **Stop** (form stays open)
- [ ] 3.3 Wire the **Stop** click: compute elapsed duration via `FormatElapsed`, set the Duration field value, save the activity, call `IActiveActivityService.Finish(activityId)`, and revert the button to show **Start**
- [ ] 3.4 When opening the edit form for an activity that is already active, render the button in its **Stop** state
- [ ] 3.5 Verify the **Start** click shows form validation errors when required fields are missing

## 4. Home Page — Active Activities Section

- [ ] 4.1 Create an `ActiveActivities` Blazor component that subscribes to `IActiveActivityService.OnChanged` and `OnTick`, renders one row per active activity showing type name and elapsed time (MMM:SS), and disposes subscriptions on unmount
- [ ] 4.2 Add a **Finish** button to each row in `ActiveActivities`; clicking it calls `IActiveActivityService.Finish`, updates the activity's Duration field via `IActivityService`, and saves the activity
- [ ] 4.3 Insert the `ActiveActivities` component into `Home.razor` after the "Activity by Goal Duration" graph and before the Activities list; hide it when no activities are active

## 5. Activities List — Finish Button

- [ ] 5.1 Inject `IActiveActivityService` into the Activities list page/component
- [ ] 5.2 Render a **Finish** button next to **Edit** for each activity row where `IActiveActivityService.IsActive(activity.Id)` is true
- [ ] 5.3 Wire the **Finish** button to compute elapsed duration, update and save the activity, and call `IActiveActivityService.Finish(activity.Id)`
- [ ] 5.4 Ensure the **Finish** button disappears from the row immediately after finishing (trigger `StateHasChanged` on `OnChanged`)

## 6. Browser Notifications for Active Activities

- [ ] 6.1 Add `startActiveNotification(activityId, name, elapsed)`, `updateActiveNotification(activityId, name, elapsed)`, and `closeActiveNotification(activityId)` to `notification-helper.js`; each uses tag `active-{activityId}` and `renotify: false, silent: true` for silent updates
- [ ] 6.2 Add an `OnSlowTick` event to `ActiveActivityService` that fires every 30 seconds (in addition to the existing 1-second `OnTick`) for driving notification updates
- [ ] 6.3 Create a headless `ActiveActivityNotification` Blazor component (renders no HTML) that subscribes to `IActiveActivityService.OnChanged` and `OnSlowTick`; on `OnChanged` it calls `startActiveNotification` / `closeActiveNotification` as appropriate; on `OnSlowTick` it calls `updateActiveNotification` for each active activity
- [ ] 6.4 Mount `ActiveActivityNotification` in `MainLayout.razor`; request notification permission via `notificationHelper.requestPermission()` on first mount

## 7. Import/Export Exclusion Verification

- [ ] 7.1 Confirm that the existing export path serializes only `Activity` records from IndexedDB and not any in-memory service state; add a comment if the exclusion is non-obvious
- [ ] 7.2 Confirm that the existing import path does not interact with `IActiveActivityService`; add a comment if needed
