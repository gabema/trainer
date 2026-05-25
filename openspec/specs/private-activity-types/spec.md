### Requirement: Activity type can be marked private
The system SHALL allow users to mark an activity type as private via the activity type create/edit form. The `ActivityType` model SHALL include an `IsPrivate` boolean field defaulting to `false`.

#### Scenario: Create new private activity type
- **WHEN** a user creates a new activity type and enables the "Private" option
- **THEN** the activity type is saved with `IsPrivate = true`

#### Scenario: Update existing activity type to be private
- **WHEN** a user edits an existing activity type and enables the "Private" option
- **THEN** the activity type is updated with `IsPrivate = true`

#### Scenario: Existing activity types default to public
- **WHEN** stored activity types without an `IsPrivate` field are loaded
- **THEN** they are treated as `IsPrivate = false` (public)

### Requirement: Private activity types hidden on home screen
The system SHALL NOT display activities whose activity type is private on the home screen, regardless of any other filters or conditions.

#### Scenario: Home screen excludes private activities
- **WHEN** the home screen loads and displays recent activities
- **THEN** any activity whose activity type has `IsPrivate = true` is not shown

#### Scenario: Home screen shows public activities normally
- **WHEN** the home screen loads and displays recent activities
- **THEN** activities whose activity type has `IsPrivate = false` are shown as before

### Requirement: Private activity types hidden on activities screen unless matched by search
The system SHALL NOT display activities whose activity type is private on the activities screen, UNLESS the active search filter term matches the private activity type's name (case-insensitive substring match).

#### Scenario: Activities screen hides private activities when no search is active
- **WHEN** the activities screen displays activities and no search term is entered
- **THEN** activities whose activity type is private are not shown

#### Scenario: Activities screen shows private activities when search matches type name
- **WHEN** the activities screen has an active search term that matches a private activity type's name
- **THEN** activities of that private type are shown in the results

#### Scenario: Activities screen hides private activities when search does not match type name
- **WHEN** the activities screen has an active search term that does not match a private activity type's name
- **THEN** activities of that private type remain hidden

### Requirement: Private activity types hidden on calendar view unless matched by search
The system SHALL NOT display activity pills for private activity types on the calendar view, UNLESS the active search filter term matches the private activity type's name (case-insensitive substring match).

#### Scenario: Calendar hides private activity pills when no search is active
- **WHEN** the calendar view renders and no search term is entered
- **THEN** activity pills for private activity types are not displayed on any day

#### Scenario: Calendar shows private activity pills when search matches type name
- **WHEN** the calendar view has an active search term matching a private activity type's name
- **THEN** activity pills for that private type appear on their respective days

#### Scenario: Calendar hides private activities when search does not match type name
- **WHEN** the calendar view has an active search term that does not match a private activity type's name
- **THEN** activity pills for that private type remain hidden

### Requirement: Private activity types remain available in activity entry
The system SHALL include private activity types in the activity type dropdown when creating or editing an activity, so users can still log activities of private types.

#### Scenario: Activity entry dropdown includes private types
- **WHEN** a user opens the create or edit activity form
- **THEN** the activity type dropdown lists all activity types, including those with `IsPrivate = true`
