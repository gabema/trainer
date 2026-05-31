## Context

The `Activity` record currently stores what, when, how much, and notes — but not where. Adding optional `Latitude`/`Longitude` fields to the model and a GPS capture button to `ActivityEntry.razor` closes this gap with minimal surface area. The app is a fully offline Blazor WASM PWA; storage is IndexedDB via JS interop.

## Goals / Non-Goals

**Goals:**
- Add optional `double? Latitude` and `double? Longitude` to `Activity`
- Add a "Use My Location" button to `ActivityEntry.razor` that calls the browser Geolocation API
- Show captured coordinates as read-only text beneath the button (lat/long with 5 decimal places)
- Allow manual text entry of coordinates as a fallback
- Persist coordinates in IndexedDB alongside the existing activity record

**Non-Goals:**
- Interactive map picker — requires map tile CDN incompatible with full offline use; deferred
- Reverse geocoding (address lookup) — out of scope
- Displaying location on existing list/calendar views
- Filtering or searching by location

## Decisions

### 1. JS Interop for Geolocation (not a .NET API)

The browser Geolocation API is JS-only. Access it via a small helper function in a new `geolocation-helper.js` file (following the existing `notification-helper.js` and `indexeddb-storage.js` pattern). The helper returns `{ latitude, longitude, error }` and is called via `IJSRuntime.InvokeAsync`.

**Alternative considered**: A third-party .NET Geolocation package (e.g., `Microsoft.AspNetCore.Components.WebAssembly.Geolocation`). Rejected — adds a NuGet dependency for a trivial JS call; rolling our own interop is 10 lines.

### 2. No map picker in v1

Leaflet.js + OpenStreetMap tiles work offline only if tiles are pre-cached, which significantly complicates the service worker. A map picker is desirable UX but not required by the core issue. Ship GPS button now; map picker is a follow-on.

### 3. Additive model change — no migration needed

`Latitude` and `Longitude` are `double?` (nullable). Existing IndexedDB records deserialize fine because JSON deserialization leaves missing fields as `null`. No IndexedDB version bump or migration script required.

### 4. Coordinate input: button + optional manual text fields

Two `<InputNumber>` fields (lat, long) pre-populated by the GPS button, but editable. This lets users correct a bad GPS fix or enter known coordinates. Fields are optional; empty = no location stored.

## Risks / Trade-offs

- **Permission denial** → Show a clear inline error message ("Location access was denied. Enable it in browser settings or enter coordinates manually."). The form remains submittable without coordinates.
- **Geolocation unavailable** (non-HTTPS, old browser) → Same error path; JS helper returns `{ error: "unavailable" }`.
- **GPS accuracy** → Raw GPS on mobile can be off by tens of meters indoors. Acceptable for this use case; accuracy is displayed to user if available.
- **PWA offline** → `navigator.geolocation` works fully offline (uses device GPS). No network required.
