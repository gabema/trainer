## 1. Build-time Version Injection

- [x] 1.1 Add an MSBuild `<Target>` to `Trainer/Trainer.csproj` that generates `obj/BuildInfo.g.cs` containing a `BuildInfo` static class with a `public const string Version` set from `$(InformationalVersion)`, defaulting to `"dev"` when unset
- [x] 1.2 Verify the generated file is excluded from source control (add `obj/` to `.gitignore` if not already present)
- [x] 1.3 Build the project and confirm `BuildInfo.Version` resolves to the expected value (e.g., `dotnet build` and grep the output or inspect the binary)

## 2. AppVersionFooter Component

- [x] 2.1 Create `Trainer/Components/AppVersionFooter.razor` that renders `<footer class="text-muted small text-center py-2">v@(BuildInfo.Version)</footer>`
- [x] 2.2 Confirm the component compiles cleanly with `dotnet build`

## 3. Page Integration

- [x] 3.1 Add `<AppVersionFooter />` at the bottom of `Trainer/Pages/Home.razor` (after all existing content, before closing tag)
- [x] 3.2 Add `<AppVersionFooter />` at the bottom of `Trainer/Pages/Activities.razor`
- [x] 3.3 Add `<AppVersionFooter />` at the bottom of `Trainer/Pages/Calendar.razor`

## 4. Verification

- [x] 4.1 Run the app locally (`dotnet run`) and visually confirm the version footer appears on the Home, Activities, and Calendar pages
- [x] 4.2 Confirm the footer does not appear on any other page (e.g., page that uses MainLayout without the component)
