## ADDED Requirements

### Requirement: Activity stores optional GPS coordinates
The `Activity` model SHALL include optional `Latitude` (`double?`) and `Longitude` (`double?`) fields. These fields SHALL default to `null` and SHALL NOT be required for saving an activity.

#### Scenario: New activity saved without location
- **WHEN** user submits the activity form with no coordinates entered
- **THEN** the activity is saved with `Latitude = null` and `Longitude = null`

#### Scenario: New activity saved with coordinates
- **WHEN** user submits the activity form with valid lat/long values populated
- **THEN** the activity is saved with the provided `Latitude` and `Longitude` values

#### Scenario: Existing activity loads with legacy coordinates
- **WHEN** an activity record in IndexedDB has no `latitude`/`longitude` fields (legacy data)
- **THEN** the activity loads successfully with `Latitude = null` and `Longitude = null`

### Requirement: GPS capture button on activity form
The activity add/edit form SHALL include a "Use My Location" button that invokes the browser Geolocation API and populates the latitude and longitude fields.

#### Scenario: GPS capture succeeds
- **WHEN** user clicks "Use My Location" and the browser grants location permission
- **THEN** the latitude and longitude fields are populated with the device's current coordinates (5 decimal places)

#### Scenario: GPS permission denied
- **WHEN** user clicks "Use My Location" and the browser denies location permission
- **THEN** an inline error message is displayed explaining that location access was denied and instructing the user to enable it in browser settings or enter coordinates manually

#### Scenario: Geolocation API unavailable
- **WHEN** user clicks "Use My Location" and the browser does not support the Geolocation API
- **THEN** an inline error message is displayed indicating that location is unavailable

#### Scenario: GPS capture in progress
- **WHEN** user clicks "Use My Location" and GPS acquisition is pending
- **THEN** the button displays a loading indicator and is disabled until acquisition completes or fails

### Requirement: Manual coordinate entry on activity form
The activity add/edit form SHALL include editable numeric input fields for latitude and longitude so users can enter or correct coordinates without using GPS.

#### Scenario: User enters coordinates manually
- **WHEN** user types valid decimal values into the latitude and longitude fields
- **THEN** those values are stored with the activity on save

#### Scenario: User clears coordinates
- **WHEN** user clears both the latitude and longitude fields
- **THEN** the activity is saved with `Latitude = null` and `Longitude = null`

### Requirement: Coordinates displayed on edit form
When editing an existing activity that has saved coordinates, the activity form SHALL pre-populate the latitude and longitude fields with the stored values.

#### Scenario: Edit activity with coordinates
- **WHEN** user opens the edit form for an activity that has `Latitude` and `Longitude` set
- **THEN** the latitude and longitude fields are pre-populated with those values
