//! Build version, replacing the `GenerateBuildInfo` MSBuild target.
//!
//! The C# target wrote a `BuildInfo.g.cs` at build time for a single line in
//! `AppVersionFooter.razor` — twenty lines of XML in `Trainer.csproj` plus a
//! generated file. Here the release workflow sets `TRAINER_VERSION` and this
//! reads it at compile time.

/// The version shown in the footer. `dev` for any build that did not come from
/// a tagged release, matching the C# target's own fallback.
pub const VERSION: &str = match option_env!("TRAINER_VERSION") {
    Some(version) => version,
    None => "dev",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_never_empty() {
        // The footer renders this directly, so an empty string would show a
        // blank footer rather than an obviously-wrong one.
        assert!(!VERSION.is_empty());
    }
}
