## Why

The `Activity` model currently stores both raw `Latitude`/`Longitude` coordinates and a `KnownLocationId`, creating redundant data and a false impression that activities can reference arbitrary GPS points. Now that every location captured on an activity is resolved to a `KnownLocation` (either matched or auto-created), the coordinate fields on `Activity` serve no purpose and add unnecessary storage weight.

## What Changes

- **BREAKING** Remove `Latitude` (double?) and `Longitude` (double?) fields from the `Activity` model and IndexedDB storage schema
- The `KnownLocationId` (int?) field remains as the sole location reference on `Activity`
- Legacy IndexedDB records with `latitude`/`longitude` but no `knownLocationId` will have those coordinate fields silently ignored on read (no migration required, fields simply not mapped)
- GPS capture on the activity form already resolves to a `KnownLocation` — no behavioral change, only model cleanup
- Any code that copies coordinates from a `KnownLocation` onto an `Activity` is removed
- **Export**: activity records in the exported JSON will no longer contain `latitude`/`longitude` fields (the serializer only writes properties that exist on the model)
- **Import**: if an import file contains `latitude`/`longitude` on activity records (exported before this change), those fields are silently ignored by the JSON deserializer — no error, no data loss

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `activity-location-capture`: Requirements must be updated — saving an activity no longer stores `Latitude`/`Longitude`; the saved-with-known-location scenario no longer populates coordinate fields on `Activity`

## Impact

- `Trainer/Models/Activity.cs` — remove `Latitude` and `Longitude` properties
- `Trainer/Pages/ActivityEntry.razor` (and code-behind) — remove any assignment of `activity.Latitude` / `activity.Longitude`
- `Trainer/Services/ActivityService.cs` — verify no coordinate reads/writes on `Activity`
- `Trainer.Tests/` — update any tests that set or assert `Latitude`/`Longitude` on `Activity`
- IndexedDB data: existing records retain the stale fields harmlessly; deserialization ignores unknown properties via camelCase JSON binding
