## 1. Update ActivitySearchFilter Helper

- [x] 1.1 Add `IReadOnlyList<KnownLocation> knownLocations` parameter to `FilterBySearch` and extend `MatchesSearch` to check location name (case-insensitive)
- [x] 1.2 Delete `FilterByLocation` method from `ActivitySearchFilter`

## 2. Update Tests

- [x] 2.1 Remove or update `ActivitySearchFilterTests` cases that reference `FilterByLocation`
- [x] 2.2 Add `FilterBySearch` tests for location-name matching (match, no match, empty list, null KnownLocationId)

## 3. Update Activities Page

- [x] 3.1 Remove the Location `<select>` and its `col-md-3` column from the Activities filter card; widen search to `col-md-9`
- [x] 3.2 Remove `_selectedLocationId` field and `OnLocationFilterChanged` handler
- [x] 3.3 Remove the `FilterByLocation` call from `GetActivitiesForDisplay` (or wherever it is applied)
- [x] 3.4 Pass `_knownLocations` to `FilterBySearch` calls in Activities page
- [x] 3.5 Simplify `DateFilterOption` enum to `AllTime` and `Custom` only (remove `Last24Hours`, `CurrentWeek`, `Last7Days`, `Last4Weeks`)
- [x] 3.6 Change `selectedDateFilter` default from `Last4Weeks` to `AllTime`
- [x] 3.7 Remove the four removed enum options from the Date Duration `<select>` markup
- [x] 3.8 Update `ApplyQueryParameters` reset path to use `AllTime` instead of `Last4Weeks`
- [x] 3.9 Update search `Placeholder` text to "Search by activity type, notes, amount, or location…"

## 4. Update Calendar Page

- [x] 4.1 Remove the Location `<select>` and its `col-md-6` column from the Calendar filter card; widen search to `col-md-12`
- [x] 4.2 Remove `_selectedLocationId` field and `OnLocationFilterChanged` handler
- [x] 4.3 Remove the `FilterByLocation` call from `GetActivitiesForDay`
- [x] 4.4 Pass `_knownLocations` to `FilterBySearch` calls in Calendar page
- [x] 4.5 Update search `Placeholder` text to "Search by activity type, notes, amount, or location…"
