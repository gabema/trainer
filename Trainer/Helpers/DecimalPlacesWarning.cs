namespace Trainer.Helpers;

/// <summary>
/// Decides whether to warn that changing an activity type's decimal precision will
/// reinterpret its existing activities (amounts are stored raw and not migrated).
/// </summary>
public static class DecimalPlacesWarning
{
    /// <summary>
    /// Warn only when the precision actually changed from the saved value AND the type
    /// already has at least one logged activity. New or empty types never warn.
    /// </summary>
    public static bool ShouldWarn(int savedDecimalPlaces, int currentDecimalPlaces, int activityCount) =>
        activityCount > 0 && currentDecimalPlaces != savedDecimalPlaces;
}
