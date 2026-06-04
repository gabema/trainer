## 1. Fix Deploy Workflow

- [x] 1.1 In `.github/workflows/deploy.yml`, update the `Build` step to strip the leading `v` from the release tag and pass it as `InformationalVersion`:
  ```yaml
  - name: Build
    run: |
      VERSION="${{ github.event.release.tag_name }}"
      dotnet build --configuration Release --no-restore -p:InformationalVersion=${VERSION#v}
  ```

## 2. Fix Footer Component

- [x] 2.1 In `Trainer/Components/AppVersionFooter.razor`, remove the hardcoded `v` prefix so the footer renders `BuildInfo.Version` exactly (e.g., change `v@(BuildInfo.Version)` to `@(BuildInfo.Version)`)

## 3. Verification

- [x] 3.1 Build locally without `InformationalVersion` set and confirm the footer shows `dev` (not `vdev`)
- [x] 3.2 Build locally with `-p:InformationalVersion=0.12.0` and confirm the footer shows `0.12.0`
