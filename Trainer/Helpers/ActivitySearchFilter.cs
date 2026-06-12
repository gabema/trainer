namespace Trainer.Helpers;

using Trainer.Models;

/// <summary>
/// Shared filter logic for activities by search term (activity type name, notes, amount, location name).
/// </summary>
public static class ActivitySearchFilter
{
    private const StringComparison SearchComparison = StringComparison.OrdinalIgnoreCase;

    /// <summary>
    /// Filters activities by search term. Matches when activity type name, notes, amount (as string), or associated
    /// known-location name contains the term (case-insensitive).
    /// Returns the input sequence unchanged when searchTerm is null, empty, or whitespace.
    /// </summary>
    public static IEnumerable<Activity> FilterBySearch(
        IEnumerable<Activity> activities,
        string? searchTerm,
        IReadOnlyList<ActivityType> activityTypes,
        IReadOnlyList<KnownLocation> knownLocations)
    {
        if (string.IsNullOrWhiteSpace(searchTerm))
        {
            return activities;
        }

        return activities.Where(a => MatchesSearch(a, searchTerm, activityTypes, knownLocations));
    }

    private static bool MatchesSearch(Activity a, string searchTerm, IReadOnlyList<ActivityType> activityTypes, IReadOnlyList<KnownLocation> knownLocations)
    {
        var activityType = activityTypes.FirstOrDefault(t => t.Id == a.ActivityTypeId);
        var typeName = activityType?.Name ?? "";
        var locationName = a.KnownLocationId.HasValue
            ? knownLocations.FirstOrDefault(l => l.Id == a.KnownLocationId.Value)?.Name ?? ""
            : "";
        var amountText = DecimalAmount.FormatDisplay(a.Amount, activityType?.DecimalPlaces ?? 0);
        return typeName.Contains(searchTerm, SearchComparison) ||
               (a.Notes ?? "").Contains(searchTerm, SearchComparison) ||
               amountText.Contains(searchTerm, SearchComparison) ||
               locationName.Contains(searchTerm, SearchComparison);
    }

    /// <summary>
    /// Removes activities whose activity type is private, unless the active search term matches the type name.
    /// When searchTerm is null/empty/whitespace, all private activities are removed.
    /// </summary>
    public static IEnumerable<Activity> FilterPrivate(
        IEnumerable<Activity> activities,
        string? searchTerm,
        IReadOnlyList<ActivityType> activityTypes)
    {
        if (string.IsNullOrWhiteSpace(searchTerm))
        {
            return activities.Where(a =>
            {
                var activityType = activityTypes.FirstOrDefault(t => t.Id == a.ActivityTypeId);
                return activityType == null || !activityType.IsPrivate;
            });
        }

        return activities.Where(a =>
        {
            var activityType = activityTypes.FirstOrDefault(t => t.Id == a.ActivityTypeId);
            if (activityType == null || !activityType.IsPrivate)
                return true;
            return activityType.Name.Contains(searchTerm, SearchComparison);
        });
    }
}
