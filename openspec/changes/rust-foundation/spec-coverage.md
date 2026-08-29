# Capability spec coverage — task 9.4

A walk through all nine existing capability specs, 41 requirements and 134
scenarios, classifying each requirement as covered by `rust-foundation` or
deferred to `rust-ui`.

`rust-foundation` ports no UI, so every requirement about a button, a page, a
navigation target, or a rendered section is deferred by design rather than
missed. What must be covered here is anything that touches the domain, the
services, or the stored bytes.

## Summary

| capability | reqs | covered here | deferred to `rust-ui` |
|---|---|---|---|
| active-activities | 5 | 2 | 3 |
| activity-duration | 2 | 2 | 0 |
| activity-filtering | 7 | 2 | 5 |
| activity-location-capture | 5 | 2 | 3 |
| app-version-footer | 3 | 0 | 3 |
| fractional-activity-amounts | 5 | 3 | 2 |
| known-locations | 9 | 6 | 3 |
| neutral-benefit | 4 | 2 | 2 |
| private-activity-types | 5 | 1 | 4 |
| **total** | **41** | **20** | **21** |

## active-activities

| requirement | status |
|---|---|
| Duration field has a Start/Stop toggle button | `rust-ui` — form control |
| Home page displays an Active Activities section | `rust-ui` — view |
| User can finish an active activity | **covered** — `services::active_activity`, `finishing_removes_an_activity_and_tolerates_unknown_ids`; the button is `rust-ui` |
| Browser notifications for active activities | `rust-ui` — `web_sys::Notification` |
| Active activity state is excluded from import/export | **covered** — `compat::active_activity_state_never_enters_an_export` |

Persistence beyond the spec's letter is also covered: the wire format, the
key-removal-when-empty rule, and silent recovery from corrupt state.

## activity-duration

| requirement | status |
|---|---|
| Duration input accepts plain minutes or M:SS | **covered** — `helpers::duration`, including `0:30`, plain minutes, and out-of-range seconds |
| Activity duration summary uses compact formatting | **covered** — `helpers::display`, including unpadded single-digit seconds |

Fully covered; only the input element itself belongs to `rust-ui`.

## activity-filtering

| requirement | status |
|---|---|
| Text search matches type name, notes, amount, location name | **covered** — `helpers::search` |
| `FilterBySearch` accepts known-locations list | **covered** — `helpers::search` |
| Date filter defaults to All Time with two options | `rust-ui` — filter control |
| Activities list shows Finish button for active activities | `rust-ui` — view |
| Keeps loading weeks under All time until results fill the view | partly — `services::week_fill` covers the loop; the view drives it |
| Scroll trigger renders whenever more weeks remain | `rust-ui` — view |
| Infinite-scroll observer re-arms after each load | `rust-ui` — `IntersectionObserver`, including the issue #85 unobserve-then-observe workaround |

## activity-location-capture

| requirement | status |
|---|---|
| Activity stores optional known location reference | **covered** — `models::Activity::known_location_id`, exercised by the real profile's 199 such activities |
| Activity export and import exclude coordinate fields | **covered** — `export_import::legacy_coordinate_fields_on_activities_are_ignored` |
| Activity form location section with picker and edit navigation | `rust-ui` — view |
| `KnownLocationEntry` page supports create and edit | `rust-ui` — page |
| GPS capture on activity form resolves to known location | partly — `find_nearby` covered; the capture button is `rust-ui` |

## app-version-footer

All three requirements are build-time wiring plus a rendered footer, and are
deferred to `rust-ui` task 1.6, which replaces the `GenerateBuildInfo` MSBuild
target with `option_env!("TRAINER_VERSION")`.

## fractional-activity-amounts

| requirement | status |
|---|---|
| Activity types define decimal precision | **covered** — `models::ActivityType::decimal_places`, both 0 and 2 present in the real profile |
| Amounts are stored as raw-scaled integers | **covered** — `helpers::amount`, and the real profile confirms every amount is an integer |
| Amounts display in decimal form | **covered** — `helpers::amount::format_display` |
| Calculator-style amount entry | partly — `extract_digits` covers the accumulator; the input element is `rust-ui` |
| Warn before reinterpreting existing activities | **covered** — `should_warn_about_decimal_places`; the dialog is `rust-ui` |

## known-locations

| requirement | status |
|---|---|
| Model stores named GPS places | **covered** |
| Service provides CRUD operations | **covered** |
| ID derived from coordinates with conflict resolution | **covered, deliberately divergent** — `HashCode.Combine` is randomly seeded per process, so nothing reproducible exists to port; ids are deterministic here and stored ids are preserved verbatim |
| Nearby lookup by GPS coordinates | **covered** — Haversine, 100 m threshold |
| Auto-name with sequential default | **covered** — including gap filling |
| Included in export and import | **covered** — round-tripped in both directions |
| Dropdown sorted alphabetically | `rust-ui` — view |
| GPS capture inline icon button | `rust-ui` — view |
| Activity card displays location name | **covered** — `helpers::display` builds the string; the card is `rust-ui` |

## neutral-benefit

| requirement | status |
|---|---|
| Neutral is the default classification | **covered** — `NetBenefit::default()` |
| Three-option selector on the form | `rust-ui` — form control |
| Neutral types appear in activity lists | **covered** — no filtering excludes them |
| Neutral types excluded from Home chart | `rust-ui` — chart |

## private-activity-types

| requirement | status |
|---|---|
| Activity type can be marked private | **covered** — `is_private`, present in the real profile |
| Hidden on home screen | `rust-ui` — view |
| Hidden on activities screen unless matched by search | **covered** — `helpers::search::filter_private` |
| Hidden on calendar view unless matched by search | `rust-ui` — view, over the same covered filter |
| Remain available in activity entry | `rust-ui` — form |

## Conclusion

Every requirement is either covered by a ported test or deferred to `rust-ui`
for an identifiable view-layer reason. Nothing is unaccounted for.

The deferred set is exactly what `rust-ui`'s task 8.1 must re-walk once views
exist; that task should treat this table as its starting checklist rather than
repeating the classification from scratch.
