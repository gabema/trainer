## MODIFIED Requirements

### Requirement: Build version is embedded at compile time
The application SHALL embed the build version string into the compiled WASM artifact via an MSBuild-generated C# constant so that the version is available without a runtime HTTP request or reflection. In CI/CD builds triggered by a GitHub Release, the release tag (with any leading `v` stripped) SHALL be passed as `$(InformationalVersion)` to the build.

#### Scenario: Version constant present after build
- **WHEN** the project is compiled
- **THEN** a `BuildInfo` class with a `Version` constant SHALL be available in the `Trainer` namespace containing a non-empty string derived from `$(InformationalVersion)`

#### Scenario: Fallback when InformationalVersion is unset
- **WHEN** the project is compiled without `$(InformationalVersion)` defined
- **THEN** the `Version` constant SHALL default to `"dev"`

#### Scenario: Release build embeds the release tag version
- **WHEN** the deploy workflow runs in response to a GitHub Release with tag `v0.12.0`
- **THEN** `BuildInfo.Version` SHALL equal `"0.12.0"` (leading `v` stripped)

### Requirement: App version footer component exists
The application SHALL provide a reusable Blazor component (`AppVersionFooter`) that renders the build version as a footer element. The component SHALL render `BuildInfo.Version` exactly, without prepending any prefix character.

#### Scenario: Footer renders version string without added prefix
- **WHEN** `BuildInfo.Version` equals `"0.12.0"` and the `AppVersionFooter` component is rendered
- **THEN** it SHALL display exactly `"0.12.0"` inside a `<footer>` element (not `"v0.12.0"`)

#### Scenario: Footer is visually unobtrusive
- **WHEN** the `AppVersionFooter` component is rendered
- **THEN** it SHALL use muted, small-sized text centered horizontally so it does not distract from page content
