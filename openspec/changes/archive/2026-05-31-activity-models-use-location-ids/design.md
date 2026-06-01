## Context

The `Activity` C# record currently has three location-related fields: `Latitude` (double?), `Longitude` (double?)`, and `KnownLocationId` (int?). The GPS capture flow on the activity form already always resolves coordinates to a `KnownLocation` before saving — either matching an existing one within 100 m or auto-creating a new one. This means `Latitude` and `Longitude` on `Activity` are always populated only as a copy of the corresponding `KnownLocation`'s coordinates, providing no additional information.

## Goals / Non-Goals

**Goals:**
- Remove `Latitude` and `Longitude` from the `Activity` model
- Ensure no code path writes those fields on `Activity` objects
- Preserve backward compatibility with existing IndexedDB data (no forced migration)

**Non-Goals:**
- Changing the `KnownLocation` model (coordinates stay there)
- Migrating legacy IndexedDB `activity` records (stale fields will be silently ignored)
- Altering the GPS capture or location-resolution flow

## Decisions

### Remove fields from the C# record only — no JS IndexedDB schema change

IndexedDB stores activities as JSON. The .NET JSON deserializer (System.Text.Json with camelCase policy) silently ignores properties in the stored JSON that have no matching C# property. Dropping `Latitude` and `Longitude` from the record is sufficient — existing records retain those bytes in IndexedDB, but they are never read. No version bump or data migration is needed.

**Alternative considered:** Bump IndexedDB version and strip coordinates from all records on upgrade. Rejected — adds complexity with zero user benefit; offline-first apps should minimise migration surface.

### No fallback from coordinates to KnownLocation during read

Legacy records that have `latitude`/`longitude` but no `knownLocationId` will load with `KnownLocationId = null` and no location displayed. A coordinate-to-location lookup on every read would be expensive and is unnecessary since the GPS capture flow already guarantees every newly saved activity uses a `KnownLocation`.

## Risks / Trade-offs

- [Legacy records lose coordinate data on next save] If a user edits an old activity and re-saves, the stale coordinates are not written back. This is acceptable — the UI already presents location via `KnownLocation` name, not raw coordinates.
- [Existing tests may set Latitude/Longitude] Tests that construct `Activity` with those fields will fail to compile after the model change. These must be updated as part of the same PR.

## Migration Plan

1. Remove `Latitude` and `Longitude` from `Activity.cs`
2. Fix all compile errors (ActivityEntry page, any test helpers)
3. Update spec for `activity-location-capture` to reflect new model contract
4. No deployment steps — PWA users get updated WASM on next load; IndexedDB data is untouched
