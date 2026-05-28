## Why

Activity logging currently captures what was done and when, but not where — losing potentially useful context for outdoor and location-dependent activities. Adding GPS capture at log time enriches the activity record without requiring manual input.

## What Changes

- Add `Latitude` and `Longitude` optional fields to the `Activity` model
- Add lat/long display fields to the Add/Edit activity form
- Add a "Use My Location" button that invokes the browser Geolocation API to populate coordinates automatically
- Add a map picker (interactive) as an alternative to GPS capture
- Store coordinates in IndexedDB alongside existing activity data

## Capabilities

### New Capabilities

- `activity-location-capture`: Browser GPS and map-based coordinate capture on the activity add/edit form, storing lat/long with each activity record

### Modified Capabilities

- `private-activity-types`: No requirement changes — implementation only touches the Activity model and form components

## Impact

- **Models**: `Activity` record gains two optional `double?` fields (`Latitude`, `Longitude`)
- **Storage**: IndexedDB schema is additive (new optional fields); existing records unaffected
- **Pages/Components**: Add/Edit activity form gains lat/long inputs and a "Use My Location" button; map picker component added
- **JS Interop**: Browser Geolocation API accessed via JS interop helper
- **Dependencies**: Leaflet.js (or similar lightweight map library) for the interactive map picker
