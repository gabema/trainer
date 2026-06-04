## Context

The deploy workflow triggers on `release: types: [created]`, giving access to `github.event.release.tag_name` (e.g., `v0.12.0`). The `GenerateBuildInfo` MSBuild target reads `$(InformationalVersion)` to write the version constant — but the workflow never sets that property, so it falls back to `"dev"`. Separately, `AppVersionFooter.razor` hardcodes a `v` prefix before the version expression.

## Goals / Non-Goals

**Goals:**
- Production builds embed the GitHub Release tag version (without a redundant `v`)
- The footer renders the version string as-is — no hardcoded prefix in the component
- Local/dev builds continue to show `dev`

**Non-Goals:**
- Changing the release tag naming convention
- Adding version to any page not already showing it

## Decisions

**1. Strip the leading `v` in the workflow shell step, not in MSBuild**

The workflow `run:` step can use bash parameter expansion `${TAG#v}` to strip a leading `v` before passing to MSBuild. This keeps the MSBuild target simple and avoids encoding shell logic in XML.

```yaml
- name: Build
  run: |
    VERSION="${{ github.event.release.tag_name }}"
    dotnet build --configuration Release --no-restore -p:InformationalVersion=${VERSION#v}
```

Alternative considered: strip in the MSBuild target with a `<PropertyGroup Condition>` — rejected because it's harder to read and test, and the workflow is the right place to normalize the input.

**2. Remove `v` prefix from the component; let the embedded string be authoritative**

`AppVersionFooter.razor` currently outputs `v@(BuildInfo.Version)`. Removing the `v` means the component renders exactly `BuildInfo.Version`. If a release tag `0.12.0` is used, the footer shows `0.12.0`. If tags use `v0.12.0` and the `#v` strip is applied, the footer still shows `0.12.0`. Consistent either way.

Alternative considered: keep the `v` in the component and never include it in the tag — rejected because it couples the component to a tagging convention and makes the local `vdev` display (two letters `v` + `dev`) look intentional when it's not.

## Risks / Trade-offs

- [Workflow-only fix] The `--no-build` flag on `dotnet publish` means the version is frozen at `dotnet build` time. If someone manually reruns just the publish step without the build, they get whatever was last built. This is an existing workflow design concern, not introduced by this change.
- [Tag format assumption] The strip `${TAG#v}` only removes a single leading `v`. Tags like `release/0.12.0` would embed `release/0.12.0` verbatim. Acceptable given current tagging practice.
