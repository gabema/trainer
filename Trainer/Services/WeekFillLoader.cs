namespace Trainer.Services;

/// <summary>
/// Drives lazy week-by-week loading until enough activities are displayed to fill the view,
/// or all available weeks have been loaded.
///
/// Used by the Activities list for the "All time + active search" case (issue #85): the search
/// filter is applied client-side over the loaded weeks, so matching activities that live in weeks
/// older than the initial batch never surface unless loading continues. A sparse filtered result
/// set also fails to make the page scrollable, so the infinite-scroll observer never re-fires.
/// </summary>
internal static class WeekFillLoader
{
    /// <summary>
    /// Selects the most-recent available week that has not yet been loaded, or <c>null</c> when
    /// every available week is already loaded.
    /// </summary>
    public static string? NextWeekKey(IEnumerable<string> availableWeekKeys, ISet<string> loadedWeekKeys)
    {
        ArgumentNullException.ThrowIfNull(availableWeekKeys);
        ArgumentNullException.ThrowIfNull(loadedWeekKeys);

        return availableWeekKeys
            .Where(weekKey => !loadedWeekKeys.Contains(weekKey))
            .OrderByDescending(weekKey => weekKey)
            .FirstOrDefault();
    }

    /// <summary>
    /// Loads successive most-recent unloaded weeks via <paramref name="loadWeek"/> until
    /// <paramref name="displayedCount"/> reports at least <paramref name="minDisplayed"/> matches,
    /// or no unloaded available week remains.
    /// </summary>
    /// <remarks>
    /// <paramref name="loadWeek"/> is responsible for loading the week's activities AND adding the
    /// week key to <paramref name="loadedWeekKeys"/>; this guarantees the loop advances and
    /// terminates (each iteration either loads a new week or breaks).
    /// </remarks>
    /// <returns>
    /// <c>true</c> when unloaded available weeks still remain afterward (i.e. there is more to load),
    /// otherwise <c>false</c>.
    /// </returns>
    public static async Task<bool> FillAsync(
        IReadOnlyCollection<string> availableWeekKeys,
        ISet<string> loadedWeekKeys,
        Func<string, Task> loadWeek,
        Func<int> displayedCount,
        int minDisplayed)
    {
        ArgumentNullException.ThrowIfNull(availableWeekKeys);
        ArgumentNullException.ThrowIfNull(loadedWeekKeys);
        ArgumentNullException.ThrowIfNull(loadWeek);
        ArgumentNullException.ThrowIfNull(displayedCount);

        while (displayedCount() < minDisplayed)
        {
            var nextWeekKey = NextWeekKey(availableWeekKeys, loadedWeekKeys);
            if (nextWeekKey is null)
            {
                break;
            }

            await loadWeek(nextWeekKey).ConfigureAwait(false);
        }

        return availableWeekKeys.Any(weekKey => !loadedWeekKeys.Contains(weekKey));
    }
}
