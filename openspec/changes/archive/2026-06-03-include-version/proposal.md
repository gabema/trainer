## Why

Users have no way to see which version of the app they are running, making it difficult to verify deployments, report bugs, or confirm they have the latest release. Displaying the build version as a footer on the main pages solves this.

## What Changes

- Capture the build version from the release branch at build time and expose it to the Blazor app
- Add a footer component that displays the version string
- Render the footer on the Home, Activities, and Calendar pages

## Capabilities

### New Capabilities
- `app-version-footer`: Displays the application build version as a footer on the Home, Activities, and Calendar pages

### Modified Capabilities

## Impact

- `Trainer/Pages/` — Home.razor, Activities.razor, Calendar.razor each get the new footer
- `Trainer/Components/` — new `AppVersionFooter` component (or equivalent inline footer)
- Build pipeline / `Trainer.csproj` — inject build version at compile time (e.g., via `<Version>` property or a generated constant)
- No API or storage changes; no breaking changes
