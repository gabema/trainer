## Why

The version footer introduced in `include-version` has two bugs: the deploy workflow never passes the release tag to the build so production always shows `vdev`, and the component prepends its own `v` to the version string meaning a `v`-prefixed tag would render as `vv0.12.0`. Both need to be fixed before the feature works correctly in production.

## What Changes

- Pass the GitHub Release tag name to `dotnet build` via `-p:InformationalVersion=` in `.github/workflows/deploy.yml`, stripping any leading `v` from the tag so the raw semver (e.g., `0.12.0`) is embedded
- Remove the hardcoded `v` prefix from `AppVersionFooter.razor` so the component renders exactly what `BuildInfo.Version` contains (the `v` prefix will come from the embedded version string if desired, or be absent if not)

## Capabilities

### New Capabilities

### Modified Capabilities
- `app-version-footer`: Version string is now sourced from the GitHub Release tag at build time; the footer renders the version exactly as embedded without an extra prepended `v`

## Impact

- `.github/workflows/deploy.yml` — `build` step gains `-p:InformationalVersion=...`
- `Trainer/Components/AppVersionFooter.razor` — removes the hardcoded `v` prefix
- No model, service, or storage changes
