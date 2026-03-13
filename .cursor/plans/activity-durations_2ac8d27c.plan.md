---
name: activity-durations
overview: Add per-activity duration tracking (in seconds) to the Trainer Blazor PWA, including storage, UI entry, and display on activity lists/cards, while keeping export/import backwards-compatible.
todos:
  - id: extend-activity-model
    content: Add nullable DurationSeconds property to the Activity model for per-activity duration in seconds.
    status: completed
  - id: update-activity-entry-ui
    content: Add and wire up a duration input on ActivityEntry.razor, mapping to Activity.DurationSeconds and updating duplication logic.
    status: completed
  - id: show-duration-in-cards-lists
    content: Update ActivityCard.razor and Index.razor amount display helpers to include duration when present.
    status: completed
  - id: verify-export-import-duration
    content: Ensure ExportImportService exports/imports DurationSeconds correctly and remains backward-compatible; update tests.
    status: completed
  - id: add-duration-roundtrip-tests
    content: Add or extend tests to verify activities with DurationSeconds round-trip through ActivityService storage logic.
    status: completed
isProject: false
---

# Add per-activity duration tracking

## Context

- Activities are represented by `Activity` in `[Trainer/Models/Activity.cs](Trainer/Models/Activity.cs)` and currently store `Id`, `ActivityTypeId`, `When`, `Amount`, and `Notes`.
- Activities are created/edited via `[Trainer/Pages/ActivityEntry.razor](Trainer/Pages/ActivityEntry.razor)` and displayed in lists/cards on `[Trainer/Pages/Index.razor](Trainer/Pages/Index.razor)` and `[Trainer/Pages/Activities.razor](Trainer/Pages/Activities.razor)` using `[Trainer/Components/ActivityCard.razor](Trainer/Components/ActivityCard.razor)`.
- Data is stored in IndexedDB via `ActivityService` and `IndexedDbStorageService`, and exported/imported as JSON via `[Trainer/Services/ExportImportService.cs](Trainer/Services/ExportImportService.cs)`, with serialization handled by `System.Text.Json` using camelCase.

## Design decisions

- **Duration unit**: Store duration as an integer number of seconds on `Activity` (e.g., `DurationSeconds`), making it optional (`int?`) so older data and non-duration activities remain valid.
- **UI entry**: Add a duration input to the Add/Edit Activity screen as a separate field from `Amount`. The UI will present this as a **single-row text entry** with a hint that the expected format is `MMM:SS` (e.g., `005:30` for 5 minutes 30 seconds), and the code will parse/format this string to/from `DurationSeconds` internally.
- **Visibility**: Show duration alongside amount on activity cards and any list/card displays, but do not change existing goal/amount-based charts or calendar aggregation logic in this iteration.
- **Compatibility**: Ensure export/import remains backward-compatible: old JSON without `durationSeconds` should still deserialize, and new exports will include the `durationSeconds` field when set.

## Implementation steps

- **1. Extend the `Activity` model**
  - Add a new nullable property `int? DurationSeconds { get; set; }` to `Activity` in `[Trainer/Models/Activity.cs](Trainer/Models/Activity.cs)`.
  - Keep it optional with no validation attributes so older serialized data and current tests continue to work without modification.
- **2. Update Add/Edit Activity UI to capture duration**
  - In `[Trainer/Pages/ActivityEntry.razor](Trainer/Pages/ActivityEntry.razor)`, add a new form field below `Amount` for duration:
    - Present it as a single-row text input with label such as "Duration (MMM:SS)".
    - Show helper text or placeholder indicating the expected `MMM:SS` format (e.g., `005:30`).
    - Bind it to a backing string property (e.g., `DurationInput`) that parses to/serializes from `activity.DurationSeconds` in seconds (handling null/empty gracefully and enforcing non-negative total seconds).
  - Ensure duplication logic (the `duplicateFrom` query handling) copies `DurationSeconds` from the source activity.
  - Optionally, add basic validation (e.g., prevent negative durations) using simple checks or data annotations if convenient.
- **3. Surface duration in activity cards and lists**
  - In `[Trainer/Components/ActivityCard.razor](Trainer/Components/ActivityCard.razor)`, update `GetAmountDisplay()` to append duration information when `Activity.DurationSeconds` has a value, e.g., "250 ml for 900s" or a more readable "for 15 min" based on simple formatting logic.
  - In `[Trainer/Pages/Index.razor](Trainer/Pages/Index.razor)`, update its local `GetAmountDisplay(Activity activity)` helper similarly so any amount text shown in the main activity list reflects duration when present.
  - Leave calendar and charts unchanged, still aggregating on `Amount` only.
- **4. Ensure export/import support and backward compatibility**
  - Verify that `ExportImportService` in `[Trainer/Services/ExportImportService.cs](Trainer/Services/ExportImportService.cs)` does not need structural changes: `Activity` now includes `DurationSeconds`, and `System.Text.Json` with camelCase will automatically handle the new `durationSeconds` field.
  - Confirm that existing import paths (array format and weekly-object format, both camelCase and PascalCase) will ignore missing `durationSeconds` for older data and populate it when present.
- **5. Update or add tests around the new field**
  - Extend `ExportImportService` tests in `[Trainer.Tests/Services/ExportImportServiceTests.cs](Trainer.Tests/Services/ExportImportServiceTests.cs)` to include a sample `Activity` with `DurationSeconds` set and assert that export JSON contains `"durationSeconds"` and that import preserves this value.
  - Optionally add a focused test in `[Trainer.Tests/Services/ActivityServiceTests.cs](Trainer.Tests/Services/ActivityServiceTests.cs)` that creates an `Activity` with `DurationSeconds` and verifies it round-trips through add/get operations unchanged.
- **6. UX polish**
  - Ensure the new duration field aligns with existing form styling and does not break mobile layout.
  - Keep labels and help text concise so the UI remains clean; consider hiding the duration field for clearly non-duration activities in a future enhancement, but for now treat it as an optional field for all activities.

