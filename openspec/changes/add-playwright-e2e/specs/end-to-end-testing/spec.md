## ADDED Requirements

### Requirement: E2E suite runs against the release deployment artifact
The end-to-end suite SHALL execute against the same Release publish output that is deployed to production, served at the `/trainer/` base path with the same SPA fallback behavior as GitHub Pages — not a development server and not a root-path build.

#### Scenario: Suite targets the sub-path artifact
- **WHEN** the E2E suite runs
- **THEN** it drives a browser against the Release-published `wwwroot` served under `/trainer/`
- **AND** assets resolve relative to the `/trainer/` base href

#### Scenario: Deep-link refresh falls back like Pages
- **WHEN** the browser requests a client-side route under `/trainer/` that is not a physical file
- **THEN** the host returns the SPA fallback document (`404.html`)
- **AND** client-side routing renders the requested view

### Requirement: Publish recipe is shared with deployment
The E2E environment and the deployment workflow SHALL produce the artifact via a single shared publish-and-rewrite procedure, so the tested artifact cannot drift from the deployed one.

#### Scenario: Both paths use one recipe
- **WHEN** the deploy workflow and the E2E job each build the artifact
- **THEN** both invoke the same shared publish script that performs the base-href, `404.html`, and `manifest.json` rewrites

### Requirement: Suite runs via dotnet test and in CI
The suite SHALL be runnable with `dotnet test` locally and SHALL run as a dedicated GitHub Actions job on pull requests, separate from the fast unit-test job.

#### Scenario: Local run with zero manual server setup
- **WHEN** a developer runs the E2E project with `dotnet test` and `E2E_BASE_URL` is not set
- **THEN** the fixture publishes and serves the artifact and runs the browser tests against it

#### Scenario: CI provides the served URL
- **WHEN** the CI job has started the static host and set `E2E_BASE_URL`
- **THEN** the suite attaches to that URL instead of self-spawning a server

### Requirement: Tests are isolated and cold-start safe
Each test SHALL run in a fresh browser context so IndexedDB state does not leak between tests, and SHALL wait on explicit application-ready signals rather than fixed delays.

#### Scenario: IndexedDB state does not leak
- **WHEN** one test writes activities to IndexedDB
- **THEN** a subsequent test starts with an empty database

#### Scenario: Waits key on a ready signal
- **WHEN** a test navigates to a page
- **THEN** it waits for a known post-boot `data-testid` element before asserting
- **AND** it does not rely on a fixed sleep or on network-idle alone

### Requirement: Suite covers the integrated JS-interop flows
The suite SHALL cover the user flows that depend on real browser JS interop — IndexedDB persistence, geolocation capture, goal charting, active-activity notification, and export/import — that unit tests cannot exercise.

#### Scenario: Persistence survives reload
- **WHEN** a user logs an activity and reloads the page
- **THEN** the activity is still present, read back from IndexedDB

#### Scenario: Geolocation capture uses mocked coordinates
- **WHEN** a test grants geolocation permission and sets coordinates
- **THEN** a location-capture flow records those coordinates

### Requirement: Failure diagnostics are captured
On test failure in CI, the suite SHALL produce Playwright traces and/or screenshots as downloadable workflow artifacts.

#### Scenario: Trace uploaded on failure
- **WHEN** an E2E test fails in CI
- **THEN** a Playwright trace (or screenshot) for the failing test is uploaded as a workflow artifact
