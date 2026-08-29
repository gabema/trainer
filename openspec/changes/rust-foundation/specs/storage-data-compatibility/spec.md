## ADDED Requirements

### Requirement: Existing browser data remains readable after the port
The Rust implementation SHALL read activity, activity type, known location, and active activity data previously written by the C# implementation without loss or transformation. A user who installed the app under the Blazor implementation SHALL see their complete history after updating, with no import step and no data migration prompt.

#### Scenario: Existing activity history loads unchanged
- **WHEN** a browser profile contains IndexedDB data written by the C# implementation and the Rust implementation loads the activity list
- **THEN** every activity is present with its original id, activity type id, timestamp, amount, notes, duration, and known location id

#### Scenario: Existing activity types load unchanged
- **WHEN** a browser profile contains a stored `activityTypes` entry written by the C# implementation
- **THEN** every activity type is present with its original id, name, net benefit, daily amount, weekly amount, unit, private flag, and decimal places

#### Scenario: Data written by Rust is readable by the previous implementation
- **WHEN** the Rust implementation writes an activity and the C# implementation subsequently reads the same store
- **THEN** the activity is parsed successfully with identical field values

### Requirement: IndexedDB value representation is preserved
Values SHALL be stored in the `activities` object store of the `Trainer` database as structured-cloned JavaScript objects, not as JSON strings. The Rust implementation SHALL parse to a JavaScript value before writing and serialize from a JavaScript value after reading, matching the boundary behavior of the existing JavaScript shim. The database name, version, and object store name SHALL be unchanged.

#### Scenario: Stored value type is an object, not a string
- **WHEN** the Rust implementation writes an activity week bucket and the raw IndexedDB record is inspected
- **THEN** the stored value is a JavaScript object, and reading it with the previous JavaScript shim returns the same JSON as before the port

#### Scenario: Database identifiers are unchanged
- **WHEN** the Rust implementation opens storage
- **THEN** it opens database `Trainer` at version 1 and uses the `activities` object store, without triggering an upgrade on an existing profile

#### Scenario: Scalar values are stored as scalars
- **WHEN** the next activity id is written to storage
- **THEN** the stored value is a JavaScript number, not an object, array, or string

### Requirement: Notes preserve three distinct states
The `notes` field SHALL distinguish absent, empty, and populated as three separate states. An empty string SHALL NOT be treated as absent. In storage all three are written explicitly; in exports the absent state is omitted while the empty state is written as `""`.

#### Scenario: Absent notes round-trip as absent
- **WHEN** an activity whose notes are absent is written and read back
- **THEN** its notes remain absent, serializing as `null` in storage and being omitted from exports

#### Scenario: Empty notes round-trip as empty
- **WHEN** an activity whose notes are an empty string is written and read back
- **THEN** its notes remain an empty string in both storage and exports, and are not converted to the absent state

### Requirement: Week bucket key format is preserved
Activities SHALL continue to be bucketed by week under keys of the form `activities-{weekKey}`, with the week key computed identically to the existing `WeekHelper`. Buckets that become empty SHALL be removed rather than left as empty arrays.

#### Scenario: Week key computation matches the previous implementation
- **WHEN** any activity timestamp from the committed fixture is passed to the Rust week key function
- **THEN** the result is character-identical to the week key the C# implementation produced for the same timestamp

#### Scenario: Emptied week buckets are removed
- **WHEN** the last activity in a week is deleted
- **THEN** the corresponding `activities-{weekKey}` entry is removed from the object store

### Requirement: Serialized JSON matches the previous serializer
Model serialization SHALL produce JSON byte-identical to `System.Text.Json` as configured in the C# implementation: camelCase property names, no indentation, field order matching the C# record declaration order, and timestamps formatted by the equivalent of the custom `DateTimeConverter`.

Empty strings SHALL be serialized as `""`. The `EmptyStringAsNullConverter` present in the C# source is dead code — registered in no `JsonSerializerOptions` and applied via no attribute — and SHALL NOT be reproduced.

#### Scenario: Fixture round-trips byte-identically
- **WHEN** the committed export fixture is deserialized into Rust models and re-serialized
- **THEN** the output is byte-identical to the fixture input

#### Scenario: Timestamp formatting is preserved
- **WHEN** an activity timestamp is serialized by the Rust implementation
- **THEN** the emitted string matches what the C# `DateTimeConverter` emitted for the same instant, including its local/UTC treatment

#### Scenario: Empty notes serialize as an empty string
- **WHEN** an activity whose notes are an empty string is serialized
- **THEN** the emitted JSON contains `"notes":""`, neither omitting the field nor writing null

### Requirement: String escaping matches the previous serializer
Serialized strings SHALL be escaped as `JavaScriptEncoder.Default` escapes them, not merely as JSON requires. A double quote SHALL be written as `"`; `&`, `'`, `+`, `<`, `>`, and a backtick SHALL be written in `\uXXXX` form; every character at U+007F or above SHALL be written in `\uXXXX` form with uppercase hexadecimal, using UTF-16 surrogate pairs above the Basic Multilingual Plane; and C0 control characters without a short form SHALL use uppercase hexadecimal.

#### Scenario: A positive UTC offset is escaped
- **WHEN** a timestamp with a positive UTC offset is serialized
- **THEN** the `+` appears as `+`, so a user east of Greenwich sees the same bytes the previous implementation wrote

#### Scenario: Note text containing HTML-sensitive characters
- **WHEN** an activity whose notes contain an apostrophe or ampersand is serialized
- **THEN** those characters appear as `'` and `&`

#### Scenario: Non-ASCII note text
- **WHEN** an activity whose notes contain a character at U+007F or above is serialized
- **THEN** that character appears in `\uXXXX` form with uppercase hexadecimal rather than as literal UTF-8

### Requirement: Export and storage use distinct serializer configurations
The implementation SHALL maintain two serializer configurations, because the C# implementation does. `ExportImportService` sets `DefaultIgnoreCondition = WhenWritingNull`, so unset optional fields are omitted from exports. `IndexedDbStorageService` sets no ignore condition, so unset optional fields are written as explicit `null` in storage. Neither configuration SHALL be used in place of the other.

#### Scenario: Export omits unset optional fields
- **WHEN** an activity with no duration is serialized for export
- **THEN** the `durationSeconds` field is absent from the output

#### Scenario: Storage writes unset optional fields as null
- **WHEN** an activity with no duration is serialized for storage
- **THEN** the output contains `"durationSeconds":null`

#### Scenario: Every offset-formatting branch is preserved
- **WHEN** timestamps are serialized under a zone with a whole-hour offset, a zone with a non-zero-minute offset, and UTC
- **THEN** the emitted strings use hour-only form for whole-hour offsets, `±hh:mm` for non-zero-minute offsets, and `Z` for zero offset, matching the committed timestamp fixtures

### Requirement: Legacy localStorage data is still migrated
The Rust implementation SHALL retain the one-time migration that moves `activities` and `activityTypes` from localStorage into IndexedDB, for users whose last use of the app predates the IndexedDB migration. Migration SHALL bucket activities by week, SHALL remove the localStorage entries only after a successful write, and SHALL NOT prevent startup if it fails.

#### Scenario: Pre-IndexedDB profile is migrated on first load
- **WHEN** a browser profile contains an `activities` entry in localStorage and no IndexedDB data
- **THEN** the activities are written to IndexedDB bucketed by week and the localStorage entry is removed

#### Scenario: Failed migration does not block startup
- **WHEN** migration raises an error
- **THEN** the error is logged and the app finishes starting up

### Requirement: Active activity state format is preserved
In-progress activities SHALL continue to persist to localStorage under the key `trainer_active_activities` as a list of entries with `id` and `startTime` fields. Corrupt stored state SHALL be cleared and treated as no active activities rather than surfacing an error.

#### Scenario: In-progress activities survive the update
- **WHEN** a user has an in-progress activity started under the C# implementation and then loads the Rust implementation
- **THEN** the activity is still shown as active with its original start time

#### Scenario: Corrupt active state is discarded silently
- **WHEN** the stored active activity state cannot be parsed
- **THEN** it is cleared and the app starts with no active activities and no error shown

### Requirement: Export and import round-trip across implementations
An export produced by the C# implementation SHALL import into the Rust implementation without loss, and an export produced by the Rust implementation SHALL import into the C# implementation without loss. The export file format SHALL be unchanged.

#### Scenario: C# export imports into Rust
- **WHEN** an export file produced by the C# implementation is imported by the Rust implementation
- **THEN** all activities, activity types, and known locations are restored with identical field values

#### Scenario: Rust export imports into C#
- **WHEN** an export file produced by the Rust implementation is imported by the C# implementation
- **THEN** all activities, activity types, and known locations are restored with identical field values
