## 1. Location Dropdown Sort

- [ ] 1.1 In `ActivityEntry.razor` `OnInitializedAsync`, sort `_knownLocations` by `Name` (case-insensitive ascending) after loading from `KnownLocationService`
- [ ] 1.2 Re-sort `_knownLocations` after `GetLocationAsync` adds a new location so the dropdown stays ordered

## 2. GPS Icon Button in Input-Group

- [ ] 2.1 Remove the standalone `<button>Use My Location</button>` block (lines 106–116 in `ActivityEntry.razor`)
- [ ] 2.2 Add a third `<button>` to the location `input-group` with `title="Get Current location"` and a location-pin SVG icon; wire its `@onclick` to `GetLocationAsync` and `disabled="@_gettingLocation"`
- [ ] 2.3 When `_gettingLocation` is true, render a `spinner-border spinner-border-sm` inside the GPS button (replacing the icon) to indicate in-progress state

## 3. Activity Amount Display Helper

- [ ] 3.1 Create `Trainer/Helpers/ActivityAmountDisplay.cs` with a static `Format(Activity activity, IEnumerable<ActivityType> activityTypes, IEnumerable<KnownLocation> knownLocations)` method that returns the formatted first-line string: `"{amount}[ {unit}][ for {duration}][ @ {locationName}]"` (each optional segment omitted when data is absent)
- [ ] 3.2 Move the duration-formatting logic currently in `ActivityCard.FormatDuration()` into `ActivityAmountDisplay` (or keep it as a private static on the helper) so the helper is self-contained
- [ ] 3.3 Write `Trainer.Tests/Helpers/ActivityAmountDisplayTests.cs` covering:
  - amount only (no unit, no duration, no location)
  - amount with unit
  - amount with duration (minutes only)
  - amount with duration (minutes and seconds)
  - amount with unit and duration
  - amount with unit, duration, and known location name
  - amount with unit and known location name (no duration)
  - `KnownLocationId` set but no matching entry in `knownLocations` list → no `" @ "` suffix
  - `KnownLocations` list is empty → no `" @ "` suffix

## 4. Activity Card Location Display

- [ ] 4.1 Add `[Parameter] public List<KnownLocation> KnownLocations { get; set; } = new();` to `ActivityCard.razor`
- [ ] 4.2 Replace the inline `GetAmountDisplay()` body in `ActivityCard.razor` with a call to `ActivityAmountDisplay.Format(Activity, ActivityTypes, KnownLocations)`
- [ ] 4.3 Add `@inject IKnownLocationService KnownLocationService` to `Index.razor` and load `_knownLocations` in `LoadData()`
- [ ] 4.4 Pass `KnownLocations="@_knownLocations"` to each `<ActivityCard>` usage in `Index.razor`
- [ ] 4.5 Add `@inject IKnownLocationService KnownLocationService` to `Activities.razor` and load `_knownLocations` alongside activities in its data-load method
- [ ] 4.6 Pass `KnownLocations="@_knownLocations"` to the `<ActivityCard>` usage in `Activities.razor`
- [ ] 4.7 Add the necessary `@using` if `KnownLocation` or `ActivityAmountDisplay` is not already in scope in `ActivityCard.razor`
