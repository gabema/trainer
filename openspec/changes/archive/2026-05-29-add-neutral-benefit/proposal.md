## Why

Some trackable activities are neither beneficial nor harmful — they're neutral habits users want to log without skewing positive/negative trends. The current `None` value conflates "unset" with "neutral," and the binary Positive/Negative toggle UI gives no way to explicitly choose neutral. Renaming `None` to `Neutral` and treating it as a real, intentional classification removes the ambiguity.

## What Changes

- Rename `NetBenefit.None` to `NetBenefit.Neutral` (same integer value `0`, backwards-compatible with stored data)
- `Neutral` is the default for new activity types and is never treated as "unset"
- Update the Create/Edit Activity Type screen to show a three-way selector (green Positive, grey Neutral, red Negative) — no deselect/reset behavior needed since Neutral is always a valid state
- Neutral activity types appear in activity lists (Home, Activities, Calendar) but are excluded from goal/benefit charts on the Home screen

## Capabilities

### New Capabilities
- `neutral-benefit`: Renames `None` → `Neutral` and promotes it to a first-class classification, including the three-way UI selector and chart exclusion logic

### Modified Capabilities

## Impact

- `Trainer/Models/NetBenefit.cs` — rename `None` to `Neutral` (value stays `0`)
- `Trainer/Pages/ActivityTypeEntry.razor` — replace two-button toggle with three-button selector; remove deselect-to-None logic
- `Trainer/Pages/Index.razor` — update chart filter from `!= None` to `!= Neutral`; add Neutral color case to color switch
- Existing stored data unaffected (integer value `0` now maps to `Neutral` instead of `None`)
