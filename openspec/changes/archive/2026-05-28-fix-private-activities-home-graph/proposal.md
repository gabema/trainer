## Why

The home page "Activity by Goal Duration" chart includes data from private activity types, leaking information that should be hidden. The activity cards section below the chart correctly filters out private activities, but the chart data pipeline skips that filter step entirely.

## What Changes

- Apply `ActivitySearchFilter.FilterPrivate()` to the activities used to build the home graph, the same way it is already applied for the activity cards display
- Extend the `private-activity-types` spec requirement to explicitly cover the goal duration chart, not just the recent activities list

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `private-activity-types`: The "hidden on home screen" requirement must explicitly cover the goal duration chart in addition to the recent-activity cards list

## Impact

- `Trainer/Pages/Index.razor` — `GetFilteredActivitiesAsync()` (used by `UpdateGoalDurationChart()`) needs privacy filtering applied before returning results
- No model, service, or API changes required; `ActivitySearchFilter.FilterPrivate()` already handles the logic
