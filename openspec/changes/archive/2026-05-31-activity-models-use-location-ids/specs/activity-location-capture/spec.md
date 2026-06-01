## MODIFIED Requirements

### Requirement: Activity stores optional known location reference
The `Activity` model SHALL include an optional `KnownLocationId` (nullable int) field. `KnownLocationId` SHALL default to `null` and SHALL NOT be required for saving an activity. The `Activity` model SHALL NOT contain `Latitude` or `Longitude` fields; coordinates are owned exclusively by the `KnownLocation` record.

#### Scenario: New activity saved without location
- **WHEN** user submits the activity form with no location selected
- **THEN** the activity is saved with `KnownLocationId = null`

#### Scenario: New activity saved with known location
- **WHEN** user selects a known location from the location picker and submits
- **THEN** the activity is saved with the selected `KnownLocationId` and no coordinate fields

#### Scenario: Existing activity loads with legacy coordinates but no KnownLocationId
- **WHEN** an activity record in IndexedDB has `latitude`/`longitude` but no `knownLocationId` (legacy data)
- **THEN** the activity loads with `KnownLocationId = null` and the stale coordinate bytes are ignored

### Requirement: Activity export and import exclude coordinate fields
The data export JSON SHALL NOT include `latitude` or `longitude` fields on activity records. When importing a JSON file that contains `latitude`/`longitude` on activity records (produced by an older version of the app), those fields SHALL be silently ignored and SHALL NOT cause an import error.

#### Scenario: Export omits coordinate fields from activities
- **WHEN** the user exports their data
- **THEN** no activity record in the resulting JSON contains a `latitude` or `longitude` field

#### Scenario: Import ignores legacy coordinate fields on activities
- **WHEN** the user imports a JSON file whose activity records include `latitude` and `longitude` fields
- **THEN** the import succeeds, activities are restored with their `knownLocationId` values, and the coordinate fields are discarded
