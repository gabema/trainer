## Why

Users need the ability to keep certain activity types hidden from general views (home, activities list, calendar) while still being able to find and log them via search. This gives users privacy control over sensitive or personal activities they track.

## What Changes

- Add an `IsPrivate` boolean field to the `ActivityType` model
- Add a "Private" toggle/checkbox to the activity type create/edit form
- Filter private activity types from the home screen activity list
- Filter private activity types from the activities screen **unless** they match the active search filter
- Filter private activity types from the calendar view **unless** they match the active search filter
- Private activity types remain selectable when creating a new activity entry (so users can still log them)

## Capabilities

### New Capabilities

- `private-activity-types`: Allows marking an activity type as private; private activities are hidden from home, activities list, and calendar views unless a search filter matches them

### Modified Capabilities

*(none — no existing specs to update)*

## Impact

- **Model**: `Trainer/Models/ActivityType.cs` — add `IsPrivate` property
- **Service**: `Trainer/Services/ActivityTypeService.cs` — existing `GetAllAsync()` returns all types; display filtering happens at the view layer
- **Pages**: `Trainer/Pages/Index.razor`, `Trainer/Pages/Activities.razor`, `Trainer/Pages/Calendar.razor` — each needs to suppress private activity types unless a search term is active and matches
- **Form**: `Trainer/Pages/ActivityTypeEntry.razor` — add private toggle UI
- **Helper**: `Trainer/Helpers/ActivitySearchFilter.cs` — may need extension to carry private-awareness through filtering logic
- **Storage**: Existing localStorage data for activity types will gain the new field; default value `false` handles backward compatibility
