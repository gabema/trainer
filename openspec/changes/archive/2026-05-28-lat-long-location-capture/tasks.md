## 1. Model

- [x] 1.1 Add `double? Latitude` and `double? Longitude` properties to `Trainer/Models/Activity.cs`
- [x] 1.2 Update the duplicate-activity block in `ActivityEntry.razor` `OnInitializedAsync` to copy `Latitude` and `Longitude` from the source activity

## 2. JS Interop

- [x] 2.1 Create `Trainer/wwwroot/js/geolocation-helper.js` with a `getLocation()` function that calls `navigator.geolocation.getCurrentPosition` and returns `{ latitude, longitude, accuracy }` or `{ error: string }` on failure/unavailability
- [x] 2.2 Register `geolocation-helper.js` in `Trainer/wwwroot/index.html` (add script tag alongside existing JS helpers)

## 3. Activity Form UI

- [x] 3.1 Add lat/long `<InputNumber>` fields and a "Use My Location" button section to `ActivityEntry.razor` below the Notes field
- [x] 3.2 Wire up the "Use My Location" button to call the JS interop helper via `IJSRuntime`, populate `activity.Latitude` / `activity.Longitude`, and display an inline error on failure
- [x] 3.3 Show a loading spinner on the button while GPS acquisition is in progress (disable button during acquisition)
- [x] 3.4 Pre-populate lat/long fields when loading an existing activity that has coordinates set (already handled by model binding once fields exist)

## 4. Tests

- [x] 4.1 Add unit tests in `Trainer.Tests` for `ActivityEntry` logic: coordinates cleared → null saved, coordinates present → values saved, duplicate-from copies coordinates
