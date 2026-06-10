namespace Trainer.Helpers;

using System.Globalization;
using Trainer.Models;

public static class ActivityAmountDisplay
{
    public static string Format(Activity activity, IEnumerable<ActivityType> activityTypes, IEnumerable<KnownLocation> knownLocations)
    {
        ArgumentNullException.ThrowIfNull(activity);

        var activityType = activityTypes.FirstOrDefault(t => t.Id == activity.ActivityTypeId);
        var amountText = activityType?.Unit != null
            ? $"{activity.Amount} {activityType.Unit}"
            : activity.Amount.ToString(CultureInfo.InvariantCulture);

        var durationText = FormatDuration(activity.DurationSeconds);
        var result = durationText != null ? $"{amountText} for {durationText}" : amountText;

        if (activity.KnownLocationId.HasValue)
        {
            var location = knownLocations.FirstOrDefault(l => l.Id == activity.KnownLocationId.Value);
            if (location != null)
                result = $"{result} @ {location.Name}";
        }

        return result;
    }

    internal static string? FormatDuration(int? durationSeconds)
    {
        if (!durationSeconds.HasValue || durationSeconds.Value <= 0)
            return null;

        var totalSeconds = durationSeconds.Value;
        var minutes = totalSeconds / 60;
        var seconds = totalSeconds % 60;

        if (minutes == 0) return $"{seconds}s";
        if (seconds == 0) return $"{minutes}m";
        return $"{minutes}m {seconds}s";
    }
}
