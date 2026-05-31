## Context

Activities currently store raw `Latitude`/`Longitude` doubles. The form shows bare coordinate inputs and a "Use My Location" GPS button. There is no concept of a named place — every activity independently holds coordinates with no shared identity across sessions. The goal is to introduce a `KnownLocation` entity so users label places once and reuse them.

## Goals / Non-Goals

**Goals:**
- `KnownLocation` model with `Id` (string GUID), `Name`, `Latitude`, `Longitude`
- `IKnownLocationService` / `KnownLocationService` backed by IndexedDB (new store `knownLocations`)
- Activity form location section becomes a known-location picker (name + optional GPS search)
- "Use My Location" resolves to the nearest known location within a threshold (e.g. 100 m); if none found, creates a new `KnownLocation` named "New Location {n}" with the captured coordinates
- `Activity` model gets optional `KnownLocationId` (nullable string)
- Known locations included in JSON export/import

**Non-Goals:**
- Map visualizations or location-based filtering/search of activities
- Merging or deduplicating existing raw coordinate data into known locations
- Map visualizations, a standalone "manage locations" settings page, or bulk editing

## Decisions

### KnownLocation ID: integer hash of initial coordinates
Compute the ID as `HashCode.Combine(latitude.GetHashCode(), longitude.GetHashCode())` (a `int`) at creation time. If that value collides with an existing location ID, increment by 1 and retry until a free slot is found. The `Activity.KnownLocationId` field is therefore an `int?` (nullable int) rather than a string.

**Why hash over coordinates:** the ID carries semantic meaning (derived from where the location was first captured), is stable across export/import, and avoids the overhead of GUID strings in IndexedDB.

**Alternative considered:** GUID string — rejected in favour of the hash approach; integer keys are simpler in IndexedDB and the hash ties the ID to the original coordinates.

**Alternative considered:** integer autoincrement — rejected because it requires a separate counter or IndexedDB autoincrement keyPath, adding JS interop complexity and producing arbitrary IDs.

### Proximity threshold for "nearby" lookup: 100 metres (configurable constant)
A static constant `NearbyThresholdMetres = 100` in `KnownLocationService`. Haversine distance computed in C#.

**Alternative considered:** expose as user setting — deferred; can be added later without schema changes.

### Activity retains raw lat/long fields
`Activity.Latitude` / `Activity.Longitude` are kept as-is. `KnownLocationId` is added additively. The form populates coordinates from the selected known location so the raw fields remain consistent for any existing consumers (export, future map features).

**Alternative considered:** remove raw fields from Activity — rejected because it is a breaking change to stored data and export format.

### "New Location {n}" naming
The service counts existing locations whose name matches `New Location \d+` and picks the next integer. Simple and deterministic without a separate counter store.

### Form UX: dropdown + navigate-to-edit (mirrors ActivityType pattern)
The activity form shows a `<select>` of known location names plus a "— No Location —" option, and an adjacent `+` / edit button — identical in structure to the ActivityType field. Clicking `+` navigates to `/known-location` (create). Clicking edit navigates to `/known-location/{id}` (edit). Both routes accept a `?returnUrl=` query parameter so the user is sent back to the activity form after saving. The `KnownLocationEntry` page (`Pages/KnownLocationEntry.razor`) handles both create and edit, showing `Name`, `Latitude`, and `Longitude` fields plus a "Use My Location" GPS button for coordinate capture. This keeps the activity form thin and reuses the established navigation pattern.

## Risks / Trade-offs

- [Legacy activities have no `KnownLocationId`] → Treated as "No location" (`null`); no migration needed.
- [Proximity matching depends on accurate GPS] → Same caveat as current GPS capture; no new risk introduced.
- [Editing a shared KnownLocation affects all activities referencing it] → Expected behavior; user is taken to a dedicated page so the scope of the edit is clear.
- [Export format changes] → New `knownLocations` top-level array added to export JSON. Import must handle missing key gracefully (treat as empty list) for backward compatibility with old exports.
