namespace Trainer.Tests.Helpers;

using Trainer.Helpers;
using Trainer.Models;

public class ActivitySearchFilterDecimalTests
{
    private static readonly List<ActivityType> ActivityTypes = new()
    {
        new() { Id = 1, Name = "Water", Unit = "L", DecimalPlaces = 2 },
    };

    private static readonly List<KnownLocation> NoLocations = new();

    private static List<Activity> Activities() => new()
    {
        new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 125 }, // shows as 1.25
        new() { Id = 2, ActivityTypeId = 1, When = DateTime.Now, Amount = 50 },  // shows as 0.50
    };

    [Fact]
    public void FilterBySearch_MatchesDecimalForm()
    {
        var result = ActivitySearchFilter.FilterBySearch(Activities(), "1.25", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterBySearch_MatchesTrimmedDecimalForm()
    {
        // Amount 50 @ 2 places displays as "0.5" (trailing zero trimmed), so that is what search matches.
        var result = ActivitySearchFilter.FilterBySearch(Activities(), "0.5", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(2, result[0].Id);
    }

    [Fact]
    public void FilterBySearch_RawIntegerDoesNotMatchScaledAmount()
    {
        // The stored integer 125 is shown as "1.25"; searching the raw "125" should not match.
        var result = ActivitySearchFilter.FilterBySearch(Activities(), "125", ActivityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }
}
