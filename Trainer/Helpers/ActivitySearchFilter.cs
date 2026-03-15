namespace Trainer.Helpers;

using System.Globalization;
using Trainer.Models;

/// <summary>
/// Shared filter logic for activities by search term (activity type name, notes, amount).
/// </summary>
public static class ActivitySearchFilter
{
    private const StringComparison SearchComparison = StringComparison.OrdinalIgnoreCase;

    /// <summary>
    /// Filters activities by search term. Matches when activity type name, notes, or amount (as string) contains the term (case-insensitive).
    /// Returns the input sequence unchanged when searchTerm is null, empty, or whitespace.
    /// </summary>
    public static IEnumerable<Activity> FilterBySearch(
        IEnumerable<Activity> activities,
        string? searchTerm,
        IReadOnlyList<ActivityType> activityTypes)
    {
        if (string.IsNullOrWhiteSpace(searchTerm))
        {
            return activities;
        }

        return activities.Where(a => MatchesSearch(a, searchTerm, activityTypes));
    }

    private static bool MatchesSearch(Activity a, string searchTerm, IReadOnlyList<ActivityType> activityTypes)
    {
        var activityType = activityTypes.FirstOrDefault(t => t.Id == a.ActivityTypeId);
        var typeName = activityType?.Name ?? "";
        return typeName.Contains(searchTerm, SearchComparison) ||
               (a.Notes ?? "").Contains(searchTerm, SearchComparison) ||
               a.Amount.ToString(CultureInfo.InvariantCulture).Contains(searchTerm, SearchComparison);
    }
}
