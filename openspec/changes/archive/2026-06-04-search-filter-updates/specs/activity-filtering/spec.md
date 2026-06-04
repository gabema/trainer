## REMOVED Requirements

### Requirement: Activities list includes a known-location filter dropdown
**Reason**: Location filtering is absorbed into the text search field. Users can type a location name directly into search.
**Migration**: Remove the Location `<select>` from the Activities filter card. Remove `_selectedLocationId` state and `OnLocationFilterChanged` handler. Remove the `FilterByLocation` call from `GetActivitiesForDisplay`.

### Requirement: Calendar view includes a known-location filter dropdown
**Reason**: Same as Activities — location filtering absorbed into text search.
**Migration**: Remove the Location `<select>` from the Calendar filter card. Remove `_selectedLocationId` state and `OnLocationFilterChanged` handler. Remove the `FilterByLocation` call from `GetActivitiesForDay`.

### Requirement: Location filter applied in ActivitySearchFilter helper
**Reason**: `FilterByLocation` is no longer called. Location matching is handled inside `FilterBySearch`.
**Migration**: Delete `ActivitySearchFilter.FilterByLocation`. Remove all call sites.

## MODIFIED Requirements

### Requirement: Activities list text search matches activity type name, notes, amount, and location name
The Activities page SHALL pass the loaded known-locations list to `ActivitySearchFilter.FilterBySearch`. `FilterBySearch` SHALL match when the activity's associated known-location name (looked up by `KnownLocationId`) contains the search term (case-insensitive). When an activity has no associated location or no matching location is found, the location name is treated as an empty string and produces no match. The search placeholder text SHALL read "Search by activity type, notes, amount, or location…".

#### Scenario: Search matches location name
- **WHEN** the user enters a search term that matches a known-location name associated with one or more activities
- **THEN** those activities are included in the results

#### Scenario: Search does not match unrelated location name
- **WHEN** the user enters a search term that does not match any field (type name, notes, amount, location name) for an activity
- **THEN** that activity is excluded from the results

#### Scenario: Activity with no location is not affected
- **WHEN** an activity has no KnownLocationId and the search term does not match type name, notes, or amount
- **THEN** that activity is excluded (location name is treated as empty, not a wildcard)

### Requirement: FilterBySearch accepts known-locations list for location name matching
`ActivitySearchFilter.FilterBySearch` SHALL accept an additional `IReadOnlyList<KnownLocation> knownLocations` parameter. When a non-empty search term is active, the method SHALL look up the activity's `KnownLocationId` in the provided list and include the location name in the match evaluation. Passing an empty list is valid and results in no location-name matches.

#### Scenario: FilterBySearch with location match
- **WHEN** `FilterBySearch` is called with a non-empty `knownLocations` list and a `searchTerm` matching a location name
- **THEN** activities associated with that location are returned

#### Scenario: FilterBySearch with empty knownLocations list
- **WHEN** `FilterBySearch` is called with an empty `knownLocations` list
- **THEN** location-name matching contributes no results; type-name, notes, and amount matching are unaffected

### Requirement: Activities date filter defaults to All Time with two options
The Activities page SHALL initialize `selectedDateFilter` to `AllTime`. The Date Duration dropdown SHALL display exactly two options: "All time" (value `AllTime`) and "Custom Range" (value `Custom`). No other date options SHALL be present. When the custom-date query parameter is cleared, the page SHALL reset `selectedDateFilter` to `AllTime`.

#### Scenario: Default filter on page load
- **WHEN** the user navigates to the Activities page with no query parameters
- **THEN** the Date Duration dropdown shows "All time" selected and activities are loaded lazily from all available weeks

#### Scenario: Date Duration dropdown shows only two options
- **WHEN** the user opens the Date Duration dropdown
- **THEN** only "All time" and "Custom Range" are listed

#### Scenario: Custom Range option shows date inputs
- **WHEN** the user selects "Custom Range"
- **THEN** Start Date and End Date inputs appear and activities are reloaded to match the selected range

#### Scenario: Reset to All Time when date query param is cleared
- **WHEN** the page previously had a date query parameter and that parameter is removed
- **THEN** selectedDateFilter resets to AllTime and all activities are lazy-loaded from scratch
