## ADDED Requirements

### Requirement: Activities list shows Finish button for active activities
The Activities list page SHALL display a **Finish** button next to the **Edit** button for any activity that is currently active. Clicking **Finish** SHALL behave identically to finishing from the Active Activities section: it computes elapsed duration, saves the activity with the updated Duration field, and removes it from the active set.

#### Scenario: Finish button visible for active activity in list
- **WHEN** an activity in the Activities list is currently active
- **THEN** a **Finish** button is displayed next to the **Edit** button for that row

#### Scenario: Finish button not visible for non-active activity
- **WHEN** an activity in the Activities list is not active
- **THEN** no **Finish** button is shown for that row

#### Scenario: Finish from Activities list updates duration and removes from active set
- **WHEN** the user clicks **Finish** on an active activity in the Activities list
- **THEN** the activity's Duration is set to elapsed time in MMM:SS format, the activity is saved, and the Finish button disappears from that row
