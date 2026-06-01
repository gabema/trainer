## 1. Model Change

- [ ] 1.1 Remove `Latitude` (double?) and `Longitude` (double?) properties from `Trainer/Models/Activity.cs`

## 2. Page Fix

- [ ] 2.1 In `Trainer/Pages/ActivityEntry.razor`, remove the two lines that copy `sourceActivity.Latitude` and `sourceActivity.Longitude` into the duplicated activity (around line 212)
- [ ] 2.2 In `Trainer/Pages/ActivityEntry.razor`, remove the two lines that assign `selectedLocation?.Latitude` and `selectedLocation?.Longitude` to `activity` before save (around line 314)

## 3. Test Updates

- [ ] 3.1 In `Trainer.Tests/Models/ActivityTests.cs`, remove or replace test cases that assert `Latitude`/`Longitude` on `Activity` — the fields no longer exist, so tests asserting default null, setting values, and copying coordinates via `with` must be removed or rewritten to verify `KnownLocationId` behaviour instead
- [ ] 3.2 Add or update export/import tests to assert that activity records in the exported JSON contain no `latitude` or `longitude` fields, and that importing a file with those fields on activities succeeds without error
