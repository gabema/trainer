## Context

`NetBenefit` is a C# enum (`None = 0`, `Positive = 1`, `Negative = 2`) on `ActivityType`. The Home page chart filters out `None` and colors bars green/red. The `ActivityTypeEntry` page has a two-button toggle (Positive / Negative) that resets to `None` on second click, making `None` serve as a hidden "unset" state.

The goal is to collapse "unset" and "neutral" into a single concept: `Neutral` is always the state if neither Positive nor Negative is chosen. No activity type is ever truly unclassified.

## Goals / Non-Goals

**Goals:**
- Rename `None` → `Neutral` (same integer value `0`, zero migration cost)
- Treat `Neutral` as a valid, intentional classification — never "unset"
- Show neutral activity types in all list views (Home recent activities, Activities page, Calendar)
- Exclude neutral activity types from the Home goal/duration chart
- Replace the two-button toggle with a three-button selector (Positive | Neutral | Negative) with no deselect behavior

**Non-Goals:**
- Any IndexedDB schema or data migration (value `0` already stored for current "None" types; rename is transparent)
- Changing chart behavior for Positive/Negative types
- Filtering neutral activities from Calendar or Activities list views

## Decisions

### 1. Rename `None → Neutral` at value `0` rather than adding a new enum member
Reusing `0` means all existing stored activity types that were `None` automatically become `Neutral` with no data migration. A new `Neutral = 3` would require a migration or leave old records misclassified. The rename is a pure refactor at the model layer.

### 2. Remove deselect-to-None logic from the form
Since `Neutral` is always a valid state (not "unset"), there is no reason to deselect. The three-button selector simply switches between the three values. This simplifies `ToggleNetBenefit` to an assignment with no toggle-off branch.

### 3. Chart filter updated from `!= None` to `!= Neutral`
The existing filter in `Index.razor` excludes `None` from chart data. Renaming the enum member is the only required change — the filter intent remains identical.

## Risks / Trade-offs

- [Reference rename] All existing references to `NetBenefit.None` must be updated to `NetBenefit.Neutral`. → Mitigation: grep for `NetBenefit.None` and `NetBenefit\.None` before shipping; the compiler will catch any missed references.
- [UI regression] Three-button layout may overflow on small screens. → Mitigation: use Bootstrap `btn-group` with responsive sizing so it reflows gracefully.
