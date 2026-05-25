## 1. Data Model

- [x] 1.1 Add `IsPrivate bool` property to `Trainer/Models/ActivityType.cs` (default `false`)

## 2. Activity Type Form

- [x] 2.1 Add a "Private" toggle/checkbox to `Trainer/Pages/ActivityTypeEntry.razor` bound to `ActivityType.IsPrivate`

## 3. Home Screen Filtering

- [x] 3.1 In `Trainer/Pages/Index.razor`, filter out activities whose activity type has `IsPrivate = true` before rendering the activity list

## 4. Activities Screen Filtering

- [x] 4.1 In `Trainer/Pages/Activities.razor`, suppress activities with a private activity type when no search term is active
- [x] 4.2 In `Trainer/Pages/Activities.razor`, show private activities only when the active search term matches their activity type name (case-insensitive substring)

## 5. Calendar View Filtering

- [x] 5.1 In `Trainer/Pages/Calendar.razor`, suppress activity pills for private activity types when no search term is active
- [x] 5.2 In `Trainer/Pages/Calendar.razor`, show private activity pills only when the active search term matches their activity type name (case-insensitive substring)

## 6. Tests

- [x] 6.1 Add unit tests in `Trainer.Tests/Helpers/ActivitySearchFilterTests.cs` covering: private types hidden with no search, private types shown when search matches, private types hidden when search does not match
