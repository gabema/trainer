## ADDED Requirements

### Requirement: Duration field has a Start/Stop toggle button
The activity form SHALL display the Duration input inside a Bootstrap input-group with a toggle button appended at the right end. The button SHALL show a timer icon and the caption **Start** when the activity is not active. Clicking **Start** SHALL validate the form, save the activity, and register it as active with the current UTC timestamp. While active, the button SHALL show a timer icon and the caption **Stop**. Clicking **Stop** SHALL compute elapsed time in MMM:SS format, write it to the Duration field, save the updated activity, and unregister it from the active set. The form SHALL remain open after both Start and Stop actions.

#### Scenario: Start button saves activity and registers it as active
- **WHEN** the user fills out the activity form and clicks the **Start** button on the Duration field
- **THEN** the activity is saved to storage, registered as active with the current UTC timestamp, and the Duration button changes to show **Stop**

#### Scenario: Stop button auto-fills duration and unregisters activity
- **WHEN** the user clicks the **Stop** button on an active activity's Duration field
- **THEN** the elapsed time is computed, the Duration field is populated with the MMM:SS value, the activity is saved, and the button reverts to showing **Start**

#### Scenario: Start button validates the form
- **WHEN** the user clicks **Start** with invalid or missing required fields
- **THEN** validation errors are displayed and no activity is saved or registered as active

#### Scenario: Duration field timer reflects active state when form reopened
- **WHEN** the user navigates to the edit form of a currently active activity
- **THEN** the Duration button shows **Stop** (reflecting the active state)

### Requirement: Home page displays an Active Activities section
The Home page SHALL display an **Active Activities** section positioned after the "Activity by Goal Duration" graph and before the main Activities list. The section SHALL only be visible when at least one activity is currently active.

#### Scenario: No active activities
- **WHEN** no activities are currently active
- **THEN** the Active Activities section is not rendered on the Home page

#### Scenario: One or more active activities shown
- **WHEN** one or more activities are active
- **THEN** the Active Activities section is visible, listing each active activity with its name/type and its current elapsed time in MMM:SS format, updated every second

### Requirement: User can finish an active activity
Each active activity entry (on the Home page Active Activities section and in the Activities list) SHALL display a **Finish** button. When clicked, the system SHALL compute the elapsed duration (MMM:SS), write it to the activity's Duration field, save the updated activity, and remove the activity from the active set.

#### Scenario: Finish button auto-fills duration
- **WHEN** the user clicks **Finish** on an active activity
- **THEN** the activity's Duration field is set to the elapsed time since Start in MMM:SS format (e.g., `002:45`), the activity is saved, and it is removed from the Active Activities section

#### Scenario: Finish removes activity from active list
- **WHEN** the last active activity is finished
- **THEN** the Active Activities section disappears from the Home page

### Requirement: Real-time elapsed time notifications for active activities
When at least one activity is active, the app SHALL display a notification message visible on all pages that lists each active activity and its current elapsed duration, updated every second.

#### Scenario: Notification appears when activity is started
- **WHEN** an activity is started
- **THEN** a notification appears showing the activity name and elapsed time (starting at 000:00)

#### Scenario: Notification updates elapsed time every second
- **WHEN** an activity is active
- **THEN** the displayed elapsed time in the notification increments by one second each second

#### Scenario: Notification disappears when all activities are finished
- **WHEN** the last active activity is finished
- **THEN** the notification is no longer shown

### Requirement: Active activity state is excluded from import/export
The active activity tracking data (activity IDs and start timestamps) SHALL NOT be included in export files and SHALL NOT be affected by import operations.

#### Scenario: Export does not include active state
- **WHEN** the user exports their data while activities are active
- **THEN** the export file contains no active-activity tracking information; active activities appear as normal (not-yet-finished) activities in the export

#### Scenario: Import does not affect active state
- **WHEN** the user imports a data file
- **THEN** the currently active activities are unaffected; no activities become active as a result of the import
