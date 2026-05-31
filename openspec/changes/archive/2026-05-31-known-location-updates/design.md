## Context

The known-locations data layer and activity form integration shipped in milestone 0.11. Three UX gaps remain:

1. The location `<select>` in `ActivityEntry.razor` renders locations in insertion order.
2. The "Use My Location" button is a full-width secondary button below the input group, inconsistent with the icon-button pattern used for activity types and the rest of the form.
3. `ActivityCard.razor` shows amount and duration but never the location name, making it impossible to distinguish activities at a glance on the Home view.

## Goals / Non-Goals

**Goals:**
- Sort the location dropdown alphabetically at load/reload time
- Replace the "Use My Location" full-width button with a compact GPS icon button appended to the location input-group (same row as the edit-pencil button)
- Display the associated known location name in the `ActivityCard` first-line summary when present

**Non-Goals:**
- Changing the `KnownLocation` model or service layer
- Adding location to the Activities list page or Calendar page cards
- Displaying coordinates or any location data beyond the name

## Decisions

### 1. Sort at the component level, not in the service

Sort `_knownLocations` after loading in `OnInitializedAsync` rather than in `KnownLocationService.GetAllAsync`. This keeps the service a neutral data layer and lets any future caller choose its own sort order.

**Alternative considered**: Sort in service — rejected to avoid imposing order on callers like KnownLocationEntry that may want a different order.

### 2. GPS button replaces the standalone button inline in the input-group

Remove the existing `<button>Use My Location</button>` and add a third button appended to the location `input-group` (after the edit-pencil button). The button renders a Bootstrap location-pin SVG icon with `title="Get Current location"` and shows a spinner inline while `_gettingLocation` is true.

**Alternative considered**: Keep the button below the input group but style it as an icon-button — rejected because it still occupies its own row, violating the issue's explicit UX spec.

### 3. Pass KnownLocations to ActivityCard as a parameter

`ActivityCard` currently receives `Activity` and `ActivityTypes`. Add a `List<KnownLocation> KnownLocations` parameter. The card's `GetAmountDisplay()` method looks up the name when `Activity.KnownLocationId` is set. Callers that don't need location (currently none, but possible) can pass an empty list.

**Alternative considered**: Inject `IKnownLocationService` into `ActivityCard` and load locations there — rejected because the card is used in a list context and per-card async calls would cause N+1 loads.

`Index.razor` and `Activities.razor` will each inject `IKnownLocationService`, load locations once during their data-load phase, and pass the list to each `<ActivityCard>` they render.

### 4. Location name appended to the `GetAmountDisplay()` string

Format: `"{amount}{unit} [for {duration}] @ {locationName}"` — the `@` separator visually distinguishes location from activity stats without needing an additional DOM element.

## Risks / Trade-offs

- [ActivityCard parameter addition] → Pages that render `<ActivityCard>` but don't pass `KnownLocations` will silently show no location name (graceful degradation — the parameter defaults to an empty list).
- [GPS button in input-group] → Three buttons in one input-group may feel cramped on narrow screens. Mitigation: the icon buttons are small (16px SVG); test on mobile viewport.
