## Why

The current activity form stores raw GPS coordinates (lat/long) but presents them as bare numbers, offering no human-readable context. This change introduces named "known locations" so users can label places they frequently log activities and quickly associate them by name rather than coordinates.

## What Changes

- Add a `KnownLocation` model with an ID, name, latitude, and longitude
- Add `IKnownLocationService` and `KnownLocationService` backed by IndexedDB
- Extend the `Activity` model with an optional `KnownLocationId` field
- Replace raw lat/long display on the activity form with a location-name selector
- "Use My Location" now searches for nearby known locations; if none found, creates a default "New Location 1" entry with editable name and coordinates
- Filter activities by known location on the Activities list and Calendar views
- Known locations are included in data export/import

## Capabilities

### New Capabilities
- `known-locations`: Persistent named locations (name + coordinates) stored in IndexedDB, selectable from the activity form; "Use My Location" resolves to or creates a known location

### Modified Capabilities
- `activity-location-capture`: Activity form location UX changes from raw coordinate inputs to a known-location picker backed by the same GPS capture flow
- `activity-filtering`: Activities list and Calendar views gain a known-location dropdown filter alongside existing search and date filters

## Impact

- New `KnownLocation` model record and `KnownLocationService` + interface
- `Activity` model gains optional `KnownLocationId` (nullable string/int)
- `ActivityForm` component (add/edit) updated to show location name instead of raw fields
- IndexedDB store additions for known locations
- Export/import JSON updated to include known locations collection
