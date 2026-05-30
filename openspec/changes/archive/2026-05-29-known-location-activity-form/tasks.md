## 1. Data Model

- [x] 1.1 Add `KnownLocation` record to `Trainer/Models/` with fields `Id` (int), `Name` (string), `Latitude` (double), `Longitude` (double)
- [x] 1.2 Add nullable `KnownLocationId` (int?) field to `Activity` model

## 2. IndexedDB Storage

- [x] 2.1 Add `knownLocations` store to the IndexedDB JS helper (`wwwroot/js/`) with keyPath `id`
- [x] 2.2 Ensure the new store is created on DB version upgrade (bump DB version)

## 3. KnownLocationService

- [x] 3.1 Create `IKnownLocationService` interface with `GetAllAsync`, `SaveAsync(KnownLocation)`, `DeleteAsync(int id)`, and `FindNearbyAsync(double lat, double lng)` methods
- [x] 3.2 Implement `KnownLocationService` backed by `IndexedDbStorageService`
- [x] 3.3 Implement ID assignment in `KnownLocationService`: compute `HashCode.Combine(lat.GetHashCode(), lng.GetHashCode())`, increment by 1 until no collision with existing IDs
- [x] 3.4 Implement Haversine distance calculation in `KnownLocationService` for 100 m proximity threshold
- [x] 3.5 Implement auto-naming logic (`"New Location {n}"`) in `KnownLocationService`
- [x] 3.6 Register `IKnownLocationService` / `KnownLocationService` as scoped in `Program.cs`

## 4. Tests

- [x] 4.1 Add `KnownLocationServiceTests` covering: save new (ID assignment from hash, collision increment), update existing, delete, find nearby (match and no-match), auto-naming (first and sequential)

## 5. KnownLocationEntry Page

- [x] 5.1 Create `Pages/KnownLocationEntry.razor` with routes `@page "/known-location"` and `@page "/known-location/{Id}"`
- [x] 5.2 Add `Name`, `Latitude`, and `Longitude` form fields; pre-populate from existing record when `Id` is provided
- [x] 5.3 Add "Use My Location" GPS button on the entry page to populate latitude/longitude fields
- [x] 5.4 On valid submit, call `KnownLocationService.SaveAsync` then navigate to `returnUrl`
- [x] 5.5 Add Cancel button that navigates to `returnUrl` without saving

## 6. Activity Form UI

- [x] 6.1 Inject `IKnownLocationService` into `ActivityEntry.razor`
- [x] 6.2 Replace raw lat/long inputs with a `<select>` dropdown listing all known locations plus "— No Location —", mirroring the ActivityType field layout
- [x] 6.3 Add an adjacent `+` / edit button: shows `+` when no location selected (navigates to `/known-location?returnUrl=...`), shows edit icon when a location is selected (navigates to `/known-location/{id}?returnUrl=...`)
- [x] 6.4 Update "Use My Location" button to call `FindNearbyAsync` after GPS capture; select match or create new location via `SaveAsync` and select it in the dropdown
- [x] 6.5 On form submit, set `Activity.KnownLocationId` and copy coordinates from selected known location (or null if none selected)

## 7. Location Filtering

- [x] 7.1 Add `FilterByLocation(IEnumerable<Activity> activities, int? locationId)` static method to `ActivitySearchFilter`
- [x] 7.2 Add `KnownLocationServiceTests` coverage for `FilterByLocation` (null → passthrough, value → filtered)
- [x] 7.3 Inject `IKnownLocationService` into `Activities.razor`; load known locations on init; add a "Location" `<select>` dropdown (All Locations + each name) to the filter card
- [x] 7.4 Add `selectedLocationId` (int?) field to `Activities.razor`; wire `OnLocationFilterChanged` handler to update it and call `ApplyFilters`
- [x] 7.5 Call `ActivitySearchFilter.FilterByLocation` inside `ApplyFilters` in `Activities.razor`
- [x] 7.6 Inject `IKnownLocationService` into `Calendar.razor`; load known locations on init; add the same "Location" `<select>` to the Calendar filter card
- [x] 7.7 Add `selectedLocationId` (int?) field to `Calendar.razor`; wire handler; apply `FilterByLocation` in `GetActivitiesForDay` (or equivalent filter pass)

## 8. Export / Import

- [x] 8.1 Add `knownLocations` array to the export JSON payload (inject `IKnownLocationService` and call `GetAllAsync`)
- [x] 8.2 Update import logic to read `knownLocations` from the JSON and upsert each via `SaveAsync`; handle missing key gracefully
