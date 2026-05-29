## 1. Model Update

- [x] 1.1 Rename `None` to `Neutral` in `Trainer/Models/NetBenefit.cs` (keep integer value `0`)
- [x] 1.2 Find and update all references to `NetBenefit.None` across the codebase (compiler errors will catch any missed ones)

## 2. Activity Type Form (UI Selector)

- [x] 2.1 Update `ActivityTypeEntry.razor` to add a Neutral button between Positive and Negative (Bootstrap `btn-secondary` / `btn-outline-secondary` styling)
- [x] 2.2 Simplify `ToggleNetBenefit` to a direct assignment — remove the toggle-off-to-None branch since Neutral is always valid
- [x] 2.3 Verify the three-button layout renders correctly on mobile viewport

## 3. Home Chart

- [x] 3.1 In `Index.razor` `UpdateGoalDurationChart`, update the `.Where` filter from `!= NetBenefit.None` to `!= NetBenefit.Neutral` (rename only — intent unchanged)
- [x] 3.2 Add a `NetBenefit.Neutral` case to the color switch in `Index.razor` (e.g., `rgba(108, 117, 125, 0.8)`) to prevent a fallthrough

## 4. Verification

- [x] 4.1 Create a Neutral activity type and confirm it appears in the Home recent-activities list and Activities page
- [x] 4.2 Confirm the Neutral activity type does NOT appear in the Home chart
- [x] 4.3 Confirm existing Positive and Negative activity types are unaffected
- [x] 4.4 Confirm a previously-stored `None` (value `0`) activity type loads correctly as Neutral
