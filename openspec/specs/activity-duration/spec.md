### Requirement: Duration input accepts plain minutes or M:SS
The activity Duration field SHALL accept either a whole number of minutes (e.g. `20`) or a colon-separated `M:SS` value (e.g. `5:30`). For sub-minute durations the user SHALL be able to enter `0:30` (zero minutes, thirty seconds); the field SHALL NOT require zero-padded minutes such as `000:30`. Minutes SHALL be in the range 0–999 and seconds in the range 0–59. An empty Duration field SHALL be treated as no duration.

#### Scenario: Sub-minute M:SS entry is accepted
- **WHEN** the user enters `0:30` in the Duration field and saves
- **THEN** the activity is saved with a duration of 30 seconds and no validation error is shown

#### Scenario: Plain minutes entry is accepted
- **WHEN** the user enters `20` in the Duration field and saves
- **THEN** the activity is saved with a duration of 20 minutes (1200 seconds)

#### Scenario: Minutes and seconds entry is accepted
- **WHEN** the user enters `5:30` in the Duration field and saves
- **THEN** the activity is saved with a duration of 5 minutes 30 seconds (330 seconds)

#### Scenario: Seconds out of range is rejected
- **WHEN** the user enters a `M:SS` value whose seconds component is 60 or greater
- **THEN** a validation error is shown and the activity is not saved

#### Scenario: Empty input means no duration
- **WHEN** the user leaves the Duration field blank and saves
- **THEN** the activity is saved with no duration

### Requirement: Activity duration summary uses compact formatting
When an activity has a duration, the system SHALL render it on activity summaries as a compact human-readable string without padding single-digit seconds: minutes-and-seconds as `Xm Ys` (e.g. `5m 5s`, `5m 30s`), whole minutes as `Xm` (e.g. `10m`), and sub-minute durations as `Ys` (e.g. `45s`). A null or non-positive duration SHALL render nothing.

#### Scenario: Single-digit seconds are not zero-padded
- **WHEN** an activity has a duration of 5 minutes 5 seconds (305 seconds)
- **THEN** its summary shows `5m 5s` (not `5m 05s`)

#### Scenario: Two-digit seconds are shown as-is
- **WHEN** an activity has a duration of 5 minutes 30 seconds (330 seconds)
- **THEN** its summary shows `5m 30s`

#### Scenario: Whole minutes omit the seconds component
- **WHEN** an activity has a duration of exactly 10 minutes (600 seconds)
- **THEN** its summary shows `10m`

#### Scenario: Sub-minute durations show seconds only
- **WHEN** an activity has a duration of 45 seconds
- **THEN** its summary shows `45s`

#### Scenario: No duration renders nothing
- **WHEN** an activity has a null or zero duration
- **THEN** no duration text is rendered for it
