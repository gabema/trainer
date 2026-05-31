### Requirement: Neutral is the default net benefit classification
`NetBenefit.Neutral` (integer value `0`, renamed from `None`) SHALL be the default value for all activity types. No activity type SHALL ever be in an "unset" state — Neutral is a valid, intentional classification meaning the activity is neither beneficial nor harmful.

#### Scenario: New activity type defaults to Neutral
- **WHEN** a user creates a new activity type without explicitly choosing a benefit
- **THEN** the activity type SHALL be saved with `NetBenefit.Neutral`

#### Scenario: Existing stored value 0 is treated as Neutral
- **WHEN** an activity type with the previously stored `None` value (`0`) is loaded
- **THEN** it SHALL be displayed and treated as `Neutral` without any data migration

### Requirement: Three-option net benefit selector on activity type form
The activity type create/edit form SHALL display three selector buttons: Positive (green), Neutral (grey), and Negative (red). The active value SHALL be visually highlighted. There SHALL be no deselect behavior — clicking any button sets that value.

#### Scenario: All three buttons visible
- **WHEN** a user opens the activity type create or edit form
- **THEN** Positive, Neutral, and Negative buttons SHALL all be displayed

#### Scenario: Active button is highlighted
- **WHEN** a user clicks the Neutral button
- **THEN** the Neutral button SHALL appear in its filled/active style and Positive/Negative buttons SHALL appear in outline style

#### Scenario: Switching from Positive to Neutral
- **WHEN** a user with Positive selected clicks Neutral
- **THEN** Neutral SHALL become active and Positive SHALL become outline

### Requirement: Neutral activity types appear in activity lists
Neutral activity types SHALL appear in all activity list views: Home recent activities, Activities page list, and Calendar view.

#### Scenario: Neutral activities shown in Home list
- **WHEN** the Home page loads recent activities
- **THEN** activities of a neutral type SHALL be included in the list

#### Scenario: Neutral activities shown in Activities page
- **WHEN** the user views the Activities list page
- **THEN** activities of a neutral type SHALL appear without suppression

### Requirement: Neutral activity types excluded from Home chart
The Home page goal/duration chart SHALL exclude activity types with `NetBenefit.Neutral`. Only `Positive` and `Negative` types SHALL be plotted.

#### Scenario: Neutral type not plotted in chart
- **WHEN** the Home chart is rendered and a neutral activity type has logged data
- **THEN** that activity type SHALL NOT appear as a bar or data point in the chart

#### Scenario: Positive and Negative still plotted
- **WHEN** the Home chart is rendered
- **THEN** activity types with Positive or Negative benefit SHALL appear in the chart as before
