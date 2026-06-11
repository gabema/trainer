### Requirement: Activity types define decimal precision
An `ActivityType` SHALL carry a decimal-precision setting that determines how many decimal places its amounts are displayed and entered with. The value MUST default to `0` and MUST be constrained to the range `0` through `3` inclusive.

#### Scenario: Default precision is whole numbers
- **WHEN** a new activity type is created without specifying precision
- **THEN** its decimal precision is `0`
- **AND** its amounts behave identically to today (no decimal point shown)

#### Scenario: Precision is bounded
- **WHEN** a user attempts to set a precision below `0` or above `3`
- **THEN** the value is rejected or clamped to the `0`–`3` range

### Requirement: Amounts are stored as raw-scaled integers
The system SHALL store `Activity.Amount` as an integer equal to the displayed value multiplied by `10^DecimalPlaces` of its activity type. No conversion or rounding SHALL occur on save beyond the integer accumulation itself, and no stored amount data SHALL be rewritten when a type's precision changes.

#### Scenario: Fractional value maps to scaled integer
- **WHEN** a user enters `1.25` for a type whose precision is `2`
- **THEN** the stored `Amount` is the integer `125`

#### Scenario: Whole-number type stores the value directly
- **WHEN** a user enters `20` for a type whose precision is `0`
- **THEN** the stored `Amount` is the integer `20`

#### Scenario: Existing data is reinterpreted, not migrated
- **WHEN** a type's precision changes from `0` to `2`
- **THEN** an existing activity stored as `Amount` `20` is now displayed as `0.20`
- **AND** the stored integer `20` is left unchanged

### Requirement: Amounts display in decimal form
The system SHALL display each activity amount as its stored integer divided by `10^DecimalPlaces` wherever amounts are shown to the user (activity cards and search matching). Read-only display SHALL drop insignificant trailing zeros from the fractional part, and the decimal point itself when no fractional digits remain. Entry fields are exempt and keep fixed precision (see "Calculator-style amount entry").

#### Scenario: Card shows decimal value
- **WHEN** an activity with stored `Amount` `125` belongs to a type with precision `2`
- **THEN** the activity card shows `1.25` (followed by the unit, if any)

#### Scenario: Trailing zeros are trimmed on display
- **WHEN** an activity with stored `Amount` `120` belongs to a type with precision `2`
- **THEN** the activity card shows `1.2`, not `1.20`

#### Scenario: A whole value drops the decimal point
- **WHEN** an activity with stored `Amount` `100` belongs to a type with precision `2`
- **THEN** the activity card shows `1`, not `1.00`

#### Scenario: Whole-number type shows no decimal point
- **WHEN** an activity with stored `Amount` `20` belongs to a type with precision `0`
- **THEN** the activity card shows `20`

### Requirement: Calculator-style amount entry
The amount entry control SHALL use calculator-style entry: a fixed field shaped to the type's precision (e.g. `0.00` for precision `2`) where typed digits accumulate into the underlying integer from the right, the decimal point is positional only, and backspace removes the most recently entered digit. The control SHALL bind directly to the raw integer amount and SHALL present a numeric keypad on mobile devices. The same control SHALL be used for the activity type's goal amount fields.

#### Scenario: Digits shift in from the right
- **WHEN** the field for a precision-`2` type starts at `0.00` and the user types `1`, then `2`, then `5`
- **THEN** the field shows `0.01`, then `0.12`, then `1.25`
- **AND** the underlying integer is `125`

#### Scenario: Backspace removes the last digit
- **WHEN** the field shows `1.25`
- **AND** the user presses backspace
- **THEN** the field shows `0.12`

#### Scenario: Clearing resets the field
- **WHEN** the user selects all or long-presses the field
- **THEN** the field resets to its zero shape (e.g. `0.00`)

#### Scenario: Goal fields use the same entry
- **WHEN** a user edits the daily or weekly goal amount for a precision-`2` type
- **THEN** the goal field uses the same calculator-style entry and stores its value at the same scale as the activity amounts

### Requirement: Warn before reinterpreting existing activities
When editing an existing activity type, the system SHALL show a warning above the decimal-places field indicating that changing it will reinterpret all existing activities of that type. The warning SHALL appear only when both the precision value differs from the saved value and the type already has at least one logged activity.

#### Scenario: Warning shown when changing precision on a type with data
- **WHEN** a user edits a type that has at least one logged activity
- **AND** changes its decimal-places value away from the saved value
- **THEN** a warning is shown above the decimal-places field

#### Scenario: No warning for a type with no activities
- **WHEN** a user edits a type that has no logged activities
- **AND** changes its decimal-places value
- **THEN** no reinterpret warning is shown

#### Scenario: No warning when the value is unchanged
- **WHEN** a user opens a type with existing activities but does not change its decimal-places value
- **THEN** no reinterpret warning is shown
