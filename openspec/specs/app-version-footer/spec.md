### Requirement: Build version is embedded at compile time
The application SHALL embed the build version string into the compiled WASM artifact via an MSBuild-generated C# constant so that the version is available without a runtime HTTP request or reflection.

#### Scenario: Version constant present after build
- **WHEN** the project is compiled
- **THEN** a `BuildInfo` class with a `Version` constant SHALL be available in the `Trainer` namespace containing a non-empty string derived from `$(InformationalVersion)`

#### Scenario: Fallback when InformationalVersion is unset
- **WHEN** the project is compiled without `$(InformationalVersion)` defined
- **THEN** the `Version` constant SHALL default to `"dev"`

### Requirement: App version footer component exists
The application SHALL provide a reusable Blazor component (`AppVersionFooter`) that renders the build version as a footer element.

#### Scenario: Footer renders version string
- **WHEN** the `AppVersionFooter` component is rendered
- **THEN** it SHALL display the text containing the version string (e.g., `v0.12.0`) inside a `<footer>` element

#### Scenario: Footer is visually unobtrusive
- **WHEN** the `AppVersionFooter` component is rendered
- **THEN** it SHALL use muted, small-sized text centered horizontally so it does not distract from page content

### Requirement: Version footer appears on Home, Activities, and Calendar pages
The Home, Activities, and Calendar pages SHALL each include the `AppVersionFooter` component at the bottom of their page content.

#### Scenario: Home page shows version footer
- **WHEN** a user navigates to the Home page
- **THEN** the page SHALL display the version footer at the bottom

#### Scenario: Activities page shows version footer
- **WHEN** a user navigates to the Activities page
- **THEN** the page SHALL display the version footer at the bottom

#### Scenario: Calendar page shows version footer
- **WHEN** a user navigates to the Calendar page
- **THEN** the page SHALL display the version footer at the bottom
