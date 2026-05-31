## Why

The initial known-locations feature shipped the data layer and basic UI wiring, but left several UX rough edges: the location dropdown is unsorted, the "Use my location" button is visually inconsistent with the rest of the form's icon-button pattern, and activity cards on the Home view show no location context at all. These three targeted fixes bring the feature to a polished, consistent state.

## What Changes

- Location dropdown in the Activity form is sorted alphabetically at render time
- "Use my location" button replaced with a compact GPS icon button placed inline next to the edit-pencil icon, with `title="Get Current location"` tooltip
- Activity card first-line summary appended with the known location name when one is associated with the activity (displayed only when present) — applies on both the Home page and the Activities page

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `known-locations`: UX requirements changing — dropdown sort order, GPS button placement/style, and activity card display of location name (Home + Activities pages) are all new behavioral requirements on top of the existing data-layer spec.

## Impact

- `Trainer/Components/` — Activity form component (dropdown sort, GPS button swap)
- `Trainer/Components/ActivityCard.razor` — location name append in amount display
- `Trainer/Pages/Index.razor` and `Trainer/Pages/Activities.razor` — load known locations once and pass to each `<ActivityCard>`
- No model, service, or IndexedDB changes required
- No breaking changes
