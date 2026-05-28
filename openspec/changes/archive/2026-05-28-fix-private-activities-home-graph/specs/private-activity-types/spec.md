## MODIFIED Requirements

### Requirement: Private activity types hidden on home screen
The system SHALL NOT display activities whose activity type is private anywhere on the home screen, including both the recent activity cards list and the "Activity by Goal Duration" chart, regardless of any other filters or conditions.

#### Scenario: Home screen activity cards exclude private activities
- **WHEN** the home screen loads and renders the recent activity cards list
- **THEN** any activity whose activity type has `IsPrivate = true` is not shown in the list

#### Scenario: Home screen chart excludes private activity types
- **WHEN** the home screen loads and renders the goal duration chart
- **THEN** any activity type with `IsPrivate = true` does not appear as a bar in the chart, regardless of the selected duration filter

#### Scenario: Home screen shows public activities normally
- **WHEN** the home screen loads
- **THEN** activities and activity types with `IsPrivate = false` are shown in both the cards list and the chart as before
