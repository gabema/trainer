### Requirement: Activities list includes a known-location filter dropdown
The Activities page SHALL display a "Location" `<select>` dropdown in the filter card alongside the existing search and date controls. The dropdown SHALL list "All Locations" as the default option followed by all stored known location names. Selecting a location filters the displayed activities to only those whose `KnownLocationId` matches the selected location. Selecting "All Locations" removes the location filter.

#### Scenario: No known locations stored
- **WHEN** the user opens the Activities page and no known locations exist
- **THEN** the Location dropdown shows only "All Locations" and applies no filter

#### Scenario: Known locations available
- **WHEN** known locations exist
- **THEN** the Location dropdown lists "All Locations" followed by each known location name

#### Scenario: Filter by location
- **WHEN** the user selects a known location from the dropdown
- **THEN** only activities whose `KnownLocationId` matches that location's `Id` are shown, in combination with any active search term and date filter

#### Scenario: Clear location filter
- **WHEN** the user selects "All Locations" from the dropdown
- **THEN** the location filter is removed and activities are shown according to the remaining active filters only

#### Scenario: Location filter combined with search term
- **WHEN** a location is selected and a search term is entered
- **THEN** only activities matching both the location and the search term are displayed

#### Scenario: Location filter combined with date range
- **WHEN** a location is selected and a date filter is active
- **THEN** only activities matching both the location and the date range are displayed

### Requirement: Calendar view includes a known-location filter dropdown
The Calendar page SHALL display the same "Location" `<select>` dropdown in its filter card. Selecting a location restricts which activity pills are shown on each calendar day cell to those matching the selected `KnownLocationId`.

#### Scenario: No known locations stored on Calendar
- **WHEN** the user opens the Calendar page and no known locations exist
- **THEN** the Location dropdown shows only "All Locations" and applies no filter

#### Scenario: Filter calendar by location
- **WHEN** the user selects a known location on the Calendar page
- **THEN** only activity pills for activities at that location are shown in the day cells; days with no matching activities show as empty

#### Scenario: Clear location filter on Calendar
- **WHEN** the user selects "All Locations" on the Calendar page
- **THEN** all activity pills are restored according to any remaining active search filter

### Requirement: Location filter applied in ActivitySearchFilter helper
`ActivitySearchFilter` SHALL expose a `FilterByLocation` method that accepts an `IEnumerable<Activity>` and a nullable `int?` locationId, returning only activities whose `KnownLocationId` equals `locationId` when a value is provided, or the full sequence when `locationId` is `null`.

#### Scenario: Filter with a location ID
- **WHEN** `FilterByLocation` is called with a non-null `locationId`
- **THEN** only activities where `KnownLocationId == locationId` are returned

#### Scenario: Filter with null location ID
- **WHEN** `FilterByLocation` is called with `locationId = null`
- **THEN** the input sequence is returned unchanged

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
