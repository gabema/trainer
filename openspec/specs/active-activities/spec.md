### Requirement: Duration field has a Start/Stop toggle button
The activity form SHALL display the Duration input inside a Bootstrap input-group with a toggle button appended at the right end. The button SHALL show a timer icon and the caption **Start** when the activity is not active. Clicking **Start** SHALL validate the form, save the activity, and register it as active with the current UTC timestamp. While active, the button SHALL show a timer icon and the caption **Stop**. Clicking **Stop** SHALL compute elapsed time in compact `M:SS` format (e.g. `2:45`, `0:30`), write it to the Duration field, save the updated activity, and unregister it from the active set. The form SHALL remain open after both Start and Stop actions.

#### Scenario: Start button saves activity and registers it as active
- **WHEN** the user fills out the activity form and clicks the **Start** button on the Duration field
- **THEN** the activity is saved to storage, registered as active using the activity's **When** field as the start time, and the Duration button changes to show **Stop**

#### Scenario: Stop button auto-fills duration and unregisters activity
- **WHEN** the user clicks the **Stop** button on an active activity's Duration field
- **THEN** the elapsed time is computed, the Duration field is populated with the compact `M:SS` value (e.g. `0:30`, not `000:30`), the activity is saved, and the button reverts to showing **Start**

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
- **THEN** the Active Activities section is visible, listing each active activity with its name/type and its current elapsed time in compact `M:SS` format, updated every second

### Requirement: User can finish an active activity
Each active activity entry (on the Home page Active Activities section and in the Activities list) SHALL display a **Finish** button. When clicked, the system SHALL compute the elapsed duration in compact `M:SS` format, write it to the activity's Duration field, save the updated activity, and remove the activity from the active set.

#### Scenario: Finish button auto-fills duration
- **WHEN** the user clicks **Finish** on an active activity
- **THEN** the activity's Duration field is set to the elapsed time since Start in compact `M:SS` format (e.g. `2:45`), the activity is saved, and it is removed from the Active Activities section

#### Scenario: Finish removes activity from active list
- **WHEN** the last active activity is finished
- **THEN** the Active Activities section disappears from the Home page

### Requirement: Browser notifications for active activities
When an activity is started, the app SHALL request browser notification permission (if not already granted) and then display a browser (OS-level) notification for that activity showing its name and elapsed time. The notification SHALL update approximately every 30 seconds with the current elapsed time using a silent replace (same tag, no re-alert sound). When the activity is finished, its notification SHALL be closed programmatically. Elapsed time in the notification SHALL use compact `M:SS` format.

#### Scenario: Permission requested on first start
- **WHEN** the user clicks Start for the first time and notification permission has not been granted
- **THEN** the browser prompts for notification permission before showing the notification

#### Scenario: Notification shown on start
- **WHEN** an activity is started and notification permission is granted
- **THEN** a browser notification appears with the activity type name as the title and the elapsed time (starting at `0:00`) in the body

#### Scenario: Notification updates elapsed time periodically
- **WHEN** an activity is active
- **THEN** the browser notification body is updated approximately every 30 seconds with the current elapsed time, silently (no re-alert sound)

#### Scenario: Notification closed on finish
- **WHEN** an activity is finished (from any surface — form, Home section, or Activities list)
- **THEN** the browser notification for that activity is closed

#### Scenario: Permission denied — no error shown
- **WHEN** the user denies notification permission
- **THEN** the activity starts and functions normally; no browser notification is shown and no error is displayed to the user

### Requirement: Active activity state is excluded from import/export
The active activity tracking data (activity IDs and start timestamps) SHALL NOT be included in export files and SHALL NOT be affected by import operations.

#### Scenario: Export does not include active state
- **WHEN** the user exports their data while activities are active
- **THEN** the export file contains no active-activity tracking information; active activities appear as normal (not-yet-finished) activities in the export

#### Scenario: Import does not affect active state
- **WHEN** the user imports a data file
- **THEN** the currently active activities are unaffected; no activities become active as a result of the import
