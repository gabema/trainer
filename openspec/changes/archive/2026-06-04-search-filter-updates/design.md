## Context

The Activities and Calendar pages each render a filter card with three controls: a text search field, a Location dropdown, and (Activities only) a Date Duration dropdown. The Location dropdown duplicates what typed search can handle and requires maintaining `_selectedLocationId` state and a separate `FilterByLocation` pass in both pages. The Date Duration dropdown offers six options, but usage patterns suggest only "All time" and "Custom range" are meaningful for this app's scope.

`ActivitySearchFilter.FilterBySearch` currently matches on activity type name, notes, and amount. `FilterByLocation` is a separate static method. Both pages call these in sequence. Both pages already load `_knownLocations` for the activity cards — that list is available for location name matching without any extra service call.

## Goals / Non-Goals

**Goals:**
- Remove the Location `<select>` from Activities and Calendar filter UI
- Extend `FilterBySearch` to match on the associated known-location name
- Default Activities date filter to AllTime (lazy-loads via existing infinite scroll)
- Reduce DateFilterOption to AllTime + Custom only
- Keep `_knownLocations` loaded (still needed for activity cards)

**Non-Goals:**
- Changing the KnownLocation model or storage
- Removing location data from existing activities
- Changing the Calendar page's date controls (Calendar has no date filter)
- Altering the search UX in any other way

## Decisions

### Extend `FilterBySearch` signature to accept known locations
`FilterBySearch` gains a `IReadOnlyList<KnownLocation> knownLocations` parameter. When a search term is active, the method also checks whether the activity's associated location name contains the term. Passing an empty list is safe and produces no location matches.

**Alternative considered**: Pre-resolve location names into a `Dictionary<int, string>` before calling. Rejected — the list is small (personal tracker), and keeping the same pattern as `activityTypes` avoids a new abstraction.

### Remove `FilterByLocation` from `ActivitySearchFilter`
Both call sites in Activities.razor and Calendar.razor will be updated to drop the `FilterByLocation` call. The method itself is deleted. No other callers exist.

**Alternative considered**: Keep `FilterByLocation` as a no-op or internal. Rejected — dead code.

### Date filter: keep only AllTime and Custom
The `DateFilterOption` enum is private to `Activities.razor`. Values `Last24Hours`, `CurrentWeek`, `Last7Days`, `Last4Weeks` are removed. Default changes from `Last4Weeks` to `AllTime`. The AllTime path already uses lazy week-key loading (8 weeks initially + infinite scroll), so performance impact is minimal.

The reset path in `ApplyQueryParameters` that resets to `Last4Weeks` is updated to reset to `AllTime`.

### Layout adjustment for Activities filter card
With Location removed, the row gains space. New layout: search `col-md-9`, date `col-md-3`. (Current: search `col-md-6`, location `col-md-3`, date `col-md-3`.)

Calendar filter card currently uses `col-md-6` for each of search and location. With location removed, search becomes `col-md-12` (or keep `col-md-6` if a second control may be added later — use full width for now).

## Risks / Trade-offs

- **Search now matches location names**: A user searching "gym" will surface activities at "My Gym" location. This is the intended behavior but may surprise users who were searching notes/type only. The filter hint text should be updated to mention "or location."
- **Removing date options**: Any user who relied on "Last 7 days" etc. must now use Custom Range. Since this is a personal single-user app the risk is low.
- **AllTime default loads lazily**: If the user scrolls fast they may see a brief spinner. This is acceptable — the issue explicitly notes lazy loading is sufficient.
