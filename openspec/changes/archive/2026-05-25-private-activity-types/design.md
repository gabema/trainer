## Context

The app stores activity types in browser localStorage/IndexedDB. The `ActivityType` model currently has: `Id`, `Name`, `NetBenefit`, `DailyAmount`, `WeeklyAmount`, `Unit`. Activities reference an activity type via `ActivityTypeId`.

Three pages display activities: home (`Index.razor`), activities list (`Activities.razor`), and calendar (`Calendar.razor`). Search filtering is centralized in `ActivitySearchFilter.FilterBySearch()`. The activities list and calendar already support a user-visible search bar; the home screen does not.

## Goals / Non-Goals

**Goals:**
- Add `IsPrivate: bool` to `ActivityType` with default `false` (backward-compatible with existing stored data)
- Suppress private activity types on the home screen entirely
- Suppress private activity types on the activities list and calendar **unless** the active search term matches the activity type name (same match logic used by `ActivitySearchFilter`)
- Expose the private flag in the activity type create/edit form
- Keep private activity types available in the activity entry dropdown so users can still log them

**Non-Goals:**
- Server-side privacy enforcement (this is a local-only app)
- Per-user or role-based access control
- Hiding individual *activities* (only the type controls visibility)
- Auditing or logging access to private activity types

## Decisions

### 1. Filter at the view layer, not the service layer

`ActivityTypeService.GetAllAsync()` continues to return all types (private and public). Each page applies its own privacy filter based on whether a search term is active.

**Rationale**: The home screen needs a blanket filter; the activities and calendar pages need context-sensitive filtering (show if search matches). Encoding that logic in the service would require passing view state into the service layer, which breaks separation of concerns. Keeping it at the view layer is simpler and consistent with how date and search filters already work.

**Alternative considered**: Add `GetPublicAsync()` to the service. Rejected because the search-visibility exception needs both private and public types available at the filtering step.

### 2. Search-visibility exception uses existing name-match logic

A private activity type becomes visible on the activities/calendar pages when the search term matches its name (case-insensitive substring). The same condition used by `ActivitySearchFilter` for name matching.

**Rationale**: Consistent with existing search behavior; no new matching rules to learn or test.

### 3. Default `IsPrivate = false` for backward compatibility

Existing stored activity types have no `IsPrivate` key. Deserialization will default to `false` (C# default for `bool`), so existing data requires no migration.

### 4. Home screen: always hidden, no search exception

The home screen has no search bar, so private types are suppressed unconditionally.

**Rationale**: Adding a search bar to the home screen is out of scope. The home screen shows recent activity at a glance; private types are omitted there by design.

## Risks / Trade-offs

- **Stale cached data**: If the browser has cached activity type lists, the new `IsPrivate` field will default to `false` after deserialization — no risk of accidental hiding. Low risk.
- **User confusion**: A private activity type that appears when a search term is active might surprise users. Mitigated by clear UI labeling (e.g., a lock icon) in the activity card or pill.
- **ActivityEntry dropdown**: Private types appear in the dropdown, which is intentional but means users can see private type names there. Acceptable — the dropdown is only shown when actively creating/editing an entry.
