namespace Trainer.Tests.Services;

using Trainer.Helpers;
using Trainer.Models;
using Trainer.Services;

public class WeekFillLoaderTests
{
    private static readonly IReadOnlyList<ActivityType> ActivityTypes = new List<ActivityType>
    {
        new() { Id = 1, Name = "Running" }
    };

    private static readonly IReadOnlyList<KnownLocation> KnownLocations = new List<KnownLocation>();

    private static Activity Match(int id, string weekKey) =>
        new() { Id = id, ActivityTypeId = 1, When = DateTime.Now, Amount = 1, Notes = $"needle {weekKey}" };

    private static Activity NoMatch(int id) =>
        new() { Id = id, ActivityTypeId = 1, When = DateTime.Now, Amount = 1, Notes = "unrelated" };

    [Fact]
    public void NextWeekKey_ReturnsMostRecentUnloadedWeek()
    {
        var available = new[] { "2026.03", "2026.01", "2026.02" };
        var loaded = new HashSet<string> { "2026.03" };

        var next = WeekFillLoader.NextWeekKey(available, loaded);

        Assert.Equal("2026.02", next);
    }

    [Fact]
    public void NextWeekKey_ReturnsNull_WhenAllWeeksLoaded()
    {
        var available = new[] { "2026.02", "2026.01" };
        var loaded = new HashSet<string> { "2026.01", "2026.02" };

        Assert.Null(WeekFillLoader.NextWeekKey(available, loaded));
    }

    [Fact]
    public async Task FillAsync_LoadsOlderWeeks_UntilMatchesSurface()
    {
        // Initial batch (newer weeks) contains no matches; the only matches live in an older week.
        var available = new[] { "2026.04", "2026.03", "2026.02", "2026.01" };
        var weekData = new Dictionary<string, List<Activity>>
        {
            ["2026.04"] = new() { NoMatch(1) },
            ["2026.03"] = new() { NoMatch(2) },
            ["2026.02"] = new() { Match(3, "2026.02"), Match(4, "2026.02") },
            ["2026.01"] = new() { NoMatch(5) }
        };

        // Simulate the two newest weeks already loaded as the initial batch.
        var loadedKeys = new HashSet<string> { "2026.04", "2026.03" };
        var accumulated = new List<Activity> { weekData["2026.04"][0], weekData["2026.03"][0] };

        var hasMore = await WeekFillLoader.FillAsync(
            available,
            loadedKeys,
            LoadWeek(weekData, accumulated, loadedKeys),
            DisplayedCount(accumulated, "needle"),
            minDisplayed: 2);

        // The older week with matches must now be loaded and its matches displayed.
        Assert.Contains("2026.02", loadedKeys);
        Assert.Equal(2, DisplayedCount(accumulated, "needle")());
        // "2026.01" remained unloaded once the threshold was met, so there is still more to load.
        Assert.DoesNotContain("2026.01", loadedKeys);
        Assert.True(hasMore);
    }

    [Fact]
    public async Task FillAsync_LoadsAllWeeks_WhenNoMatchesExistAnywhere()
    {
        // None of the weeks contain a match — fill must exhaust every week and report no more.
        var available = new[] { "2026.03", "2026.02", "2026.01" };
        var weekData = new Dictionary<string, List<Activity>>
        {
            ["2026.03"] = new() { NoMatch(1) },
            ["2026.02"] = new() { NoMatch(2) },
            ["2026.01"] = new() { NoMatch(3) }
        };

        var loadedKeys = new HashSet<string>();
        var accumulated = new List<Activity>();

        var hasMore = await WeekFillLoader.FillAsync(
            available,
            loadedKeys,
            LoadWeek(weekData, accumulated, loadedKeys),
            DisplayedCount(accumulated, "needle"),
            minDisplayed: 1);

        Assert.Equal(available.ToHashSet(), loadedKeys);
        Assert.False(hasMore);
        Assert.Equal(0, DisplayedCount(accumulated, "needle")());
    }

    [Fact]
    public async Task FillAsync_DoesNotLoad_WhenThresholdAlreadyMet()
    {
        var available = new[] { "2026.02", "2026.01" };
        var weekData = new Dictionary<string, List<Activity>>
        {
            ["2026.02"] = new() { Match(1, "2026.02") },
            ["2026.01"] = new() { Match(2, "2026.01") }
        };

        var loadedKeys = new HashSet<string> { "2026.02" };
        var accumulated = new List<Activity> { weekData["2026.02"][0] };

        var hasMore = await WeekFillLoader.FillAsync(
            available,
            loadedKeys,
            LoadWeek(weekData, accumulated, loadedKeys),
            DisplayedCount(accumulated, "needle"),
            minDisplayed: 1);

        // Threshold already met, so no further week is loaded, but one remains available.
        Assert.DoesNotContain("2026.01", loadedKeys);
        Assert.True(hasMore);
    }

    private static Func<string, Task> LoadWeek(
        Dictionary<string, List<Activity>> weekData,
        List<Activity> accumulated,
        HashSet<string> loadedKeys) =>
        weekKey =>
        {
            accumulated.AddRange(weekData[weekKey]);
            loadedKeys.Add(weekKey);
            return Task.CompletedTask;
        };

    private static Func<int> DisplayedCount(List<Activity> accumulated, string searchTerm) =>
        () => ActivitySearchFilter
            .FilterBySearch(accumulated, searchTerm, ActivityTypes, KnownLocations)
            .Count();
}
