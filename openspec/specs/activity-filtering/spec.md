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

### Requirement: Activities list shows Finish button for active activities
The Activities list page SHALL display a **Finish** button next to the **Edit** button for any activity that is currently active. Clicking **Finish** SHALL behave identically to finishing from the Active Activities section: it computes elapsed duration, saves the activity with the updated Duration field, and removes it from the active set.

#### Scenario: Finish button visible for active activity in list
- **WHEN** an activity in the Activities list is currently active
- **THEN** a **Finish** button is displayed next to the **Edit** button for that row

#### Scenario: Finish button not visible for non-active activity
- **WHEN** an activity in the Activities list is not active
- **THEN** no **Finish** button is shown for that row

#### Scenario: Finish from Activities list updates duration and removes from active set
- **WHEN** the user clicks **Finish** on an active activity in the Activities list
- **THEN** the activity's Duration is set to elapsed time in MMM:SS format, the activity is saved, and the Finish button disappears from that row

### Requirement: Activities list keeps loading weeks under All time until search results fill the view
When the Activities date filter is `AllTime` and a non-empty search term is active, the Activities page SHALL continue loading additional available weeks (oldest-remaining after the initial batch) until either the number of displayed (search-filtered) activities is sufficient to make the page scrollable OR all available weeks have been loaded. The page SHALL NOT cap search results to the initially loaded weeks. Loading SHALL stop once no unloaded available week remains, at which point `hasMoreWeeksToLoad` is false.

#### Scenario: Matches exist only in older weeks
- **WHEN** the date filter is All time, a search term is active, and matching activities exist only in weeks older than the initial batch
- **THEN** the page loads successive older weeks until those matching activities are displayed (or all weeks are exhausted), rather than stopping at the initial batch

#### Scenario: Initial weeks contain no matches
- **WHEN** the date filter is All time, a search term is active, and none of the initially loaded weeks contain a matching activity
- **THEN** the page continues loading older weeks while showing the loading indicator, and only shows "No activities found" once all available weeks are loaded and still produce no matches

#### Scenario: All weeks exhausted
- **WHEN** every available week has been loaded
- **THEN** `hasMoreWeeksToLoad` is false and no further load attempts are made

### Requirement: Scroll trigger renders whenever more weeks remain to load
The Activities page SHALL render the infinite-scroll trigger element whenever `hasMoreWeeksToLoad` is true, independent of whether the current displayed (search-filtered) result set is empty. When the displayed result set is empty but unloaded weeks remain, the page SHALL keep the trigger and/or loading indicator present so that lazy loading can continue.

#### Scenario: Trigger present with empty filtered results
- **WHEN** the displayed result set is currently empty and more weeks remain to load
- **THEN** the scroll trigger element is rendered (and observed) so loading continues

#### Scenario: Trigger absent when fully loaded
- **WHEN** all available weeks have been loaded
- **THEN** the scroll trigger element is not rendered

### Requirement: Infinite-scroll observer re-arms after each load
After each week-loading cycle completes and the component re-renders, the Activities page SHALL re-observe the scroll trigger element while `hasMoreWeeksToLoad` is true, so that a trigger that remains within the viewport continues to drive subsequent loads instead of stalling.

#### Scenario: Sparse results keep trigger in view
- **WHEN** a load completes, the filtered results do not fill the viewport, and the scroll trigger remains visible with more weeks to load
- **THEN** the observer is re-armed and the next week is loaded without requiring the user to scroll
