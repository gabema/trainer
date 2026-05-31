## 1. Fix Chart Privacy Filter

- [x] 1.1 In `Trainer/Pages/Index.razor`, update `GetFilteredActivitiesAsync()` to apply `ActivitySearchFilter.FilterPrivate(filtered, null, activityTypes)` before returning, mirroring `GetFilteredActivitiesForDisplay()`

## 2. Tests

- [x] 2.1 Add a unit test in `Trainer.Tests` for `Index.razor` (or an equivalent helper) verifying that activities with a private `ActivityType` are excluded from the data returned for the chart
