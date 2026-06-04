## Context

The app is a Blazor WebAssembly PWA with no server component. There is currently no mechanism to surface the build version to the user. The version needs to be injected at build time and made available to Razor components without a runtime API call.

## Goals / Non-Goals

**Goals:**
- Inject the assembly/package version into the compiled WASM artifact at build time
- Display it as a small, unobtrusive footer on the Home, Activities, and Calendar pages
- Require zero configuration changes at runtime

**Non-Goals:**
- Git commit SHA display
- Dynamic version checking / update notifications
- Version shown on any page other than Home, Activities, and Calendar

## Decisions

**1. Version source: `AssemblyInformationalVersion` via `IConfiguration` or a generated constant**

Use a C# `partial class` with a `const string` generated from `$(InformationalVersion)` via an MSBuild `<Target>` that writes a `BuildInfo.g.cs` file into the project's `obj/` directory. This keeps the version purely in C# without needing `appsettings.json` or JS interop.

Alternative considered: reading `appsettings.json` — rejected because it requires a separate asset file and an extra HTTP round-trip on first load.

Alternative considered: `Assembly.GetExecutingAssembly().GetCustomAttribute<AssemblyInformationalVersionAttribute>()` — viable but more verbose and reflection-based; a generated constant is simpler and AOT-friendly.

**2. UI placement: shared `AppVersionFooter` Razor component**

A reusable component (`Trainer/Components/AppVersionFooter.razor`) renders a `<footer>` tag with the version string. Each of the three pages includes `<AppVersionFooter />` at the bottom of its markup. This avoids duplicating the footer HTML across three files while keeping it lightweight (no service injection needed).

Alternative considered: putting the footer in `MainLayout.razor` — rejected because it would appear on every page (e.g., Settings, Login-style screens) rather than just the three specified pages.

**3. Styling: Bootstrap utility classes only**

The footer uses `text-muted small text-center py-2` Bootstrap classes. No dedicated `.razor.css` scoped stylesheet is needed for such minimal styling.

## Risks / Trade-offs

- [Build pipeline coupling] The generated `BuildInfo.g.cs` depends on the MSBuild `$(InformationalVersion)` property being set correctly. If the property is missing, the version will be empty/default. → Mitigation: fall back to `"dev"` when the property is empty using a Condition in the MSBuild target.
- [Three-page update] Three pages must each add `<AppVersionFooter />`. Small blast radius; straightforward find-and-add.
