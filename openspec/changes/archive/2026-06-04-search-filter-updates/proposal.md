## Why

The Activities and Calendar filter bars have accumulated controls that add friction without meaningful value: a separate Location dropdown duplicates what text search can handle, and the Date Duration picker exposes five options where users only need two. Simplifying these controls reduces visual clutter and aligns the default state with how most users browse (all time, filtered by keyword).

## What Changes

- **Remove** the Location `<select>` dropdown from the Activities page filter card
- **Remove** the Location `<select>` dropdown from the Calendar page filter card
- **Extend** the text search to match on associated known-location name (so location filtering is absorbed into the existing search field)
- **Change** the Activities page default date filter from "Last 4 Weeks" to "All Time"
- **Reduce** the Date Duration dropdown to two options only: "All Time" (default) and "Custom Range" (removing Last 24 Hours, Current Week, Last 7 Days, Last 4 Weeks)
- **Remove** the `FilterByLocation` method from `ActivitySearchFilter` (no longer needed as a standalone filter step)
- **Remove** the `_selectedLocationId` state and `OnLocationFilterChanged` handler from Activities and Calendar pages

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `activity-filtering`: Remove location dropdown requirement from Activities and Calendar pages; extend text search to match location names; change date filter default to AllTime and reduce options to AllTime + Custom only; remove FilterByLocation from ActivitySearchFilter.

## Impact

- `Trainer/Pages/Activities.razor` — remove location UI + state, change `selectedDateFilter` default, simplify `DateFilterOption` enum, update layout
- `Trainer/Pages/Calendar.razor` — remove location UI + state
- `Trainer/Helpers/ActivitySearchFilter.cs` — remove `FilterByLocation`, extend `FilterBySearch` to also match on known-location name
- `openspec/specs/activity-filtering/spec.md` — delta: retire location-dropdown requirements, update search requirements to include location name, update date-filter requirements
- `Trainer.Tests` — update or remove tests for `FilterByLocation`; add tests for location-name text search
