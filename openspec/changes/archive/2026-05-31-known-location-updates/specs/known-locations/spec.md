## ADDED Requirements

### Requirement: Location dropdown is sorted alphabetically
The Activity form SHALL display known locations in the location `<select>` sorted alphabetically by `Name` (case-insensitive, ascending) at the time the list is loaded.

#### Scenario: Locations render in alphabetical order
- **WHEN** the Activity form loads and multiple known locations exist
- **THEN** the location dropdown options appear in ascending alphabetical order by name

#### Scenario: Single location renders without change
- **WHEN** only one known location exists
- **THEN** that location appears as the single option (sort has no visible effect)

### Requirement: GPS capture uses an inline icon button in the location input-group
The Activity form's location field SHALL present the GPS capture action as a compact icon button appended to the location input-group (same row as the edit-pencil button), replacing the standalone full-width "Use My Location" button. The button SHALL carry the `title` attribute `"Get Current location"` and display a GPS/location pin icon. While location acquisition is in progress the button SHALL show a small spinner and be disabled; error messages SHALL still appear beneath the input-group.

#### Scenario: GPS button appears in the input-group
- **WHEN** the Activity form renders the location field
- **THEN** the GPS icon button is the third button in the location input-group and no standalone "Use My Location" button exists outside the input-group

#### Scenario: GPS button shows spinner while acquiring
- **WHEN** the user taps the GPS icon button and location acquisition is in progress
- **THEN** the button displays a spinner and is disabled until acquisition completes or fails

#### Scenario: GPS button tooltip
- **WHEN** the user hovers over the GPS icon button
- **THEN** a tooltip reading "Get Current location" is displayed

### Requirement: Activity card displays associated known location name
`ActivityCard` SHALL accept a `KnownLocations` parameter (`List<KnownLocation>`, defaulting to empty). When the activity's `KnownLocationId` is set and a matching `KnownLocation` exists in the provided list, the first-line summary SHALL append `" @ {locationName}"` after the amount/duration text. When no location is associated or no match is found, the first-line summary is unchanged.

#### Scenario: Card shows location name when present
- **WHEN** an activity has a `KnownLocationId` and the matching location is in the `KnownLocations` list
- **THEN** the amount display reads `"{amount}{unit} [for {duration}] @ {locationName}"`

#### Scenario: Card omits location when not set
- **WHEN** an activity has no `KnownLocationId`
- **THEN** the amount display shows only amount and optional duration, with no `" @ "` suffix

#### Scenario: Home page passes known locations to each card
- **WHEN** the Home page renders the activity list
- **THEN** all known locations are loaded once and passed to every `<ActivityCard>` instance

#### Scenario: Activities page passes known locations to each card
- **WHEN** the Activities page renders the activity list
- **THEN** all known locations are loaded once and passed to every `<ActivityCard>` instance

## MODIFIED Requirements

### Requirement: KnownLocationService provides CRUD operations
`IKnownLocationService` SHALL expose `GetAllAsync`, `SaveAsync(KnownLocation)`, `DeleteAsync(int id)`, `FindNearbyAsync(double latitude, double longitude)`, and `NextAutoNameAsync()` operations. `KnownLocationService` SHALL implement this interface and be registered as a scoped service in `Program.cs`. `GetAllAsync` returns records in storage order; callers are responsible for any sorting needed for display.

#### Scenario: Save new known location
- **WHEN** `SaveAsync` is called with a `KnownLocation` whose `Id` is `0` (default)
- **THEN** a hash-derived integer ID is assigned and the record is stored in IndexedDB

#### Scenario: Update existing known location
- **WHEN** `SaveAsync` is called with a `KnownLocation` whose `Id` already exists in IndexedDB
- **THEN** the existing record is overwritten with the new values

#### Scenario: Delete known location
- **WHEN** `DeleteAsync` is called with a valid `Id`
- **THEN** the record is removed from the `knownLocations` store

#### Scenario: GetAllAsync returns all records
- **WHEN** `GetAllAsync` is called
- **THEN** all stored `KnownLocation` records are returned (order is storage-defined)
