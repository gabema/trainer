## Context

The home page has two sections that consume activity data: the "Activity by Goal Duration" chart and the recent activity cards list. Both use activities loaded in `LoadData()` via `ActivityService.GetAllAsync()`.

The cards list calls `GetFilteredActivitiesForDisplay()`, which already applies `ActivitySearchFilter.FilterPrivate(filtered, null, activityTypes)`. The chart calls `GetFilteredActivitiesAsync()`, which only date-filters — no privacy step.

## Goals / Non-Goals

**Goals:**
- Private activity types must not appear in the home page chart, matching the behavior already in place for the activity cards

**Non-Goals:**
- Changes to ActivitySearchFilter, ActivityService, or any model
- Changes to any other page (Activities list and Calendar already handle privacy correctly)

## Decisions

**Apply the privacy filter inside `GetFilteredActivitiesAsync()`** rather than inside `UpdateGoalDurationChart()`.

Rationale: `GetFilteredActivitiesAsync()` is the single data-preparation method used by the chart. Filtering there keeps the call site clean and makes the method safe to call from any future chart logic without each caller needing to remember to re-apply the privacy filter. The alternative — filtering inside `UpdateGoalDurationChart()` — would duplicate the concern and leave `GetFilteredActivitiesAsync()` misleadingly returning private data.

The call mirrors what `GetFilteredActivitiesForDisplay()` already does: pass `null` as the search term (no search is active on the home page) and the loaded `activityTypes` list.

## Risks / Trade-offs

- **[Risk] `activityTypes` not yet loaded when chart initializes** → Mitigation: `LoadData()` already loads `activityTypes` before calling `UpdateGoalDurationChart()`, so the list will always be populated when the filter runs.
- **[Risk] Changing `GetFilteredActivitiesAsync()` affects callers added in future** → Low risk: the method is private to `Index.razor`; there are no other callers.
