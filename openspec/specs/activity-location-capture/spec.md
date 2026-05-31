### Requirement: Activity stores optional known location reference
The `Activity` model SHALL include an optional `KnownLocationId` (nullable string) field in addition to the existing `Latitude` and `Longitude` fields. `KnownLocationId` SHALL default to `null` and SHALL NOT be required for saving an activity.

#### Scenario: New activity saved without location
- **WHEN** user submits the activity form with no location selected
- **THEN** the activity is saved with `KnownLocationId = null`, `Latitude = null`, and `Longitude = null`

#### Scenario: New activity saved with known location
- **WHEN** user selects a known location from the location picker and submits
- **THEN** the activity is saved with the selected `KnownLocationId` and the known location's `Latitude` and `Longitude`

#### Scenario: Existing activity loads with legacy coordinates but no KnownLocationId
- **WHEN** an activity record in IndexedDB has `latitude`/`longitude` but no `knownLocationId` (legacy data)
- **THEN** the activity loads with those coordinates and `KnownLocationId = null`

### Requirement: Activity form location section shows known-location picker with edit navigation
The activity add/edit form SHALL display a `<select>` dropdown listing all known location names plus a "— No Location —" option, and an adjacent button. When no location is selected the button shows `+` and navigates to `/known-location?returnUrl=<encoded-return-url>` to create a new known location. When a location is selected the button shows an edit icon and navigates to `/known-location/{id}?returnUrl=<encoded-return-url>` to edit that location. This mirrors the ActivityType field pattern on the same form.

#### Scenario: No known locations exist
- **WHEN** the user opens the activity form and no known locations are stored
- **THEN** the dropdown shows only "— No Location —" and the adjacent button shows `+`

#### Scenario: Known locations available
- **WHEN** the user opens the activity form and known locations exist
- **THEN** the dropdown lists all known location names in addition to "— No Location —"

#### Scenario: Selecting a known location shows edit button
- **WHEN** the user selects a known location from the dropdown
- **THEN** the adjacent button changes to an edit icon

#### Scenario: Create new known location via + button
- **WHEN** no location is selected and the user clicks `+`
- **THEN** the user is navigated to `/known-location?returnUrl=<encoded-return-url>`

#### Scenario: Edit selected known location via edit button
- **WHEN** a known location is selected and the user clicks the edit button
- **THEN** the user is navigated to `/known-location/{id}?returnUrl=<encoded-return-url>`

#### Scenario: Return to activity form after editing location
- **WHEN** the user saves or cancels on the `KnownLocationEntry` page
- **THEN** the user is navigated back to the activity form via the `returnUrl`

#### Scenario: Selecting No Location
- **WHEN** the user selects "— No Location —" from the dropdown
- **THEN** the activity will be saved with `KnownLocationId = null`

### Requirement: KnownLocationEntry page supports create and edit of a known location
A Blazor page at route `/known-location` (create) and `/known-location/{Id}` (edit) SHALL allow the user to set `Name`, `Latitude`, and `Longitude` for a `KnownLocation`. The page SHALL accept a `returnUrl` query parameter and navigate to it after a successful save or on cancel.

#### Scenario: Create new known location
- **WHEN** the user navigates to `/known-location` and submits the form with a valid name and coordinates
- **THEN** a new `KnownLocation` is saved via `KnownLocationService` and the user is navigated to `returnUrl`

#### Scenario: Edit existing known location
- **WHEN** the user navigates to `/known-location/{id}` and modifies the name or coordinates and submits
- **THEN** the existing `KnownLocation` is updated and the user is navigated to `returnUrl`

#### Scenario: Cancel returns to activity form
- **WHEN** the user clicks Cancel on the `KnownLocationEntry` page
- **THEN** the user is navigated to `returnUrl` without saving

#### Scenario: GPS capture on KnownLocationEntry page
- **WHEN** the user clicks "Use My Location" on the `KnownLocationEntry` page
- **THEN** the latitude and longitude fields are populated with the device's current coordinates

### Requirement: GPS capture on activity form resolves to known location
The "Use My Location" button on the activity form SHALL invoke the browser Geolocation API, then search for a nearby known location within 100 m. If a match is found, it SHALL be selected in the dropdown. If no match is found, a new `KnownLocation` SHALL be created with the captured coordinates and an auto-generated name, then selected in the dropdown.

#### Scenario: GPS capture finds nearby known location
- **WHEN** user clicks "Use My Location" and the captured coordinates are within 100 m of an existing known location
- **THEN** that known location is selected in the dropdown

#### Scenario: GPS capture finds no nearby location
- **WHEN** user clicks "Use My Location" and no known location is within 100 m
- **THEN** a new `KnownLocation` is created with the captured coordinates and an auto-generated name, and it is selected in the dropdown

#### Scenario: GPS permission denied
- **WHEN** user clicks "Use My Location" and the browser denies location permission
- **THEN** an inline error message is displayed explaining that location access was denied

#### Scenario: Geolocation API unavailable
- **WHEN** user clicks "Use My Location" and the browser does not support the Geolocation API
- **THEN** an inline error message is displayed indicating that location is unavailable

#### Scenario: GPS capture in progress
- **WHEN** user clicks "Use My Location" and GPS acquisition is pending
- **THEN** the button displays a loading indicator and is disabled until acquisition completes or fails
