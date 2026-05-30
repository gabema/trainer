### Requirement: KnownLocation model stores named GPS places
The system SHALL maintain a `KnownLocation` record with fields `Id` (int), `Name` (string), `Latitude` (double), and `Longitude` (double). Known locations SHALL be persisted in a dedicated IndexedDB store named `knownLocations` with an integer keyPath.

#### Scenario: Known location saved to IndexedDB
- **WHEN** a `KnownLocation` is created or updated via `KnownLocationService`
- **THEN** it is persisted in the `knownLocations` IndexedDB store and retrievable by its `Id`

#### Scenario: All known locations retrieved
- **WHEN** `KnownLocationService.GetAllAsync()` is called
- **THEN** it returns all stored `KnownLocation` records

### Requirement: KnownLocationService provides CRUD operations
`IKnownLocationService` SHALL expose `GetAllAsync`, `SaveAsync(KnownLocation)`, and `DeleteAsync(int id)` operations. `KnownLocationService` SHALL implement this interface and be registered as a scoped service in `Program.cs`.

#### Scenario: Save new known location
- **WHEN** `SaveAsync` is called with a `KnownLocation` whose `Id` is `0` (default)
- **THEN** a hash-derived integer ID is assigned (see ID assignment requirement) and the record is stored in IndexedDB

#### Scenario: Update existing known location
- **WHEN** `SaveAsync` is called with a `KnownLocation` whose `Id` already exists in IndexedDB
- **THEN** the existing record is overwritten with the new values

#### Scenario: Delete known location
- **WHEN** `DeleteAsync` is called with a valid `Id`
- **THEN** the record is removed from the `knownLocations` store

### Requirement: KnownLocation ID is derived from initial coordinates with conflict resolution
When a new `KnownLocation` is created, `KnownLocationService` SHALL compute the ID as `HashCode.Combine(latitude.GetHashCode(), longitude.GetHashCode())`. If that integer is already used by an existing location, the service SHALL increment the candidate by 1 and retry until a free value is found.

#### Scenario: Hash produces unique ID
- **WHEN** a new `KnownLocation` is created and the hash of its coordinates does not match any existing ID
- **THEN** the hash value is used as the `Id`

#### Scenario: Hash collision resolved by increment
- **WHEN** a new `KnownLocation` is created and the hash of its coordinates matches an existing ID
- **THEN** the service increments the candidate ID by 1 and retries until a free slot is found, using that value as the `Id`

### Requirement: Nearby known location lookup by GPS coordinates
`KnownLocationService` SHALL provide a `FindNearbyAsync(double latitude, double longitude)` method that returns the closest `KnownLocation` within 100 metres using the Haversine formula, or `null` if none qualify.

#### Scenario: Nearby location found
- **WHEN** `FindNearbyAsync` is called with coordinates within 100 m of an existing known location
- **THEN** that known location is returned

#### Scenario: No nearby location found
- **WHEN** `FindNearbyAsync` is called with coordinates more than 100 m from all known locations
- **THEN** `null` is returned

### Requirement: Auto-name new locations with sequential default name
When a new `KnownLocation` is created programmatically (via GPS capture with no nearby match), the system SHALL assign the name `"New Location {n}"` where `n` is the lowest integer not already used by an existing known location name matching that pattern.

#### Scenario: First auto-named location
- **WHEN** no known locations exist with a name matching `New Location \d+`
- **THEN** the new location is named `"New Location 1"`

#### Scenario: Sequential auto-naming
- **WHEN** `"New Location 1"` already exists and a second auto-named location is created
- **THEN** the new location is named `"New Location 2"`

### Requirement: Known locations included in export and import
The data export JSON SHALL include a top-level `knownLocations` array containing all `KnownLocation` records. Import SHALL restore known locations from this array, treating a missing `knownLocations` key as an empty list for backward compatibility.

#### Scenario: Export includes known locations
- **WHEN** the user exports their data
- **THEN** the resulting JSON contains a `knownLocations` array with all stored locations

#### Scenario: Import restores known locations
- **WHEN** the user imports a JSON file containing a `knownLocations` array
- **THEN** all locations in that array are upserted into IndexedDB

#### Scenario: Import from legacy export without knownLocations key
- **WHEN** the user imports a JSON file that does not contain a `knownLocations` key
- **THEN** the import succeeds and no known locations are created or modified
