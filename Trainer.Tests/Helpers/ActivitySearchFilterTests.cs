namespace Trainer.Tests.Helpers;

using Trainer.Helpers;
using Trainer.Models;

public class ActivitySearchFilterTests
{
    private static readonly List<ActivityType> ActivityTypes = new()
    {
        new() { Id = 1, Name = "Running" },
        new() { Id = 2, Name = "Swimming" },
        new() { Id = 3, Name = "Reading" },
    };

    private static readonly List<KnownLocation> NoLocations = new();

    private static readonly List<KnownLocation> KnownLocations = new()
    {
        new() { Id = 10, Name = "My Gym", Latitude = 0, Longitude = 0 },
        new() { Id = 20, Name = "Home Pool", Latitude = 0, Longitude = 0 },
    };

    [Fact]
    public void FilterBySearch_NullSearch_ReturnsInputUnchanged()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 10, Notes = "run" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, null, ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterBySearch_EmptySearch_ReturnsInputUnchanged()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 10, Notes = "run" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
    }

    [Theory]
    [InlineData(" ")]
    [InlineData("   ")]
    public void FilterBySearch_WhitespaceSearch_ReturnsInputUnchanged(string searchTerm)
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 10, Notes = "run" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, searchTerm, ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_MatchByActivityTypeName_IncludesMatchingActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 0, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "Run", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
        Assert.Equal(1, result[0].ActivityTypeId);
    }

    [Fact]
    public void FilterBySearch_MatchByNotes_IncludesMatchingActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "quick morning run" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 0, Notes = "pool session" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "morning", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterBySearch_MatchByAmount_IncludesMatchingActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 15, Notes = "" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 20, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "15", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(15, result[0].Amount);
    }

    [Fact]
    public void FilterBySearch_AmountSubstringMatch_IncludesMatchingActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 150, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "15", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(150, result[0].Amount);
    }

    [Theory]
    [InlineData("RUN")]
    [InlineData("run")]
    [InlineData("Running")]
    public void FilterBySearch_CaseInsensitiveTypeName_MatchesRegardlessOfCase(string searchTerm)
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, searchTerm, ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_CaseInsensitiveNotes_MatchesRegardlessOfCase()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "my NOTE here" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "note", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_NoMatches_ReturnsEmpty()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "run" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "xyz", ActivityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_MultipleActivities_ReturnsOnlyMatching()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 0, Notes = "" },
            new() { Id = 3, ActivityTypeId = 3, When = DateTime.Now, Amount = 0, Notes = "read a book" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "read", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        Assert.Equal(3, result[0].Id); // Only "Reading" type name contains "read"
        Assert.DoesNotContain(result, a => a.Id == 1);
        Assert.DoesNotContain(result, a => a.Id == 2);
    }

    [Fact]
    public void FilterBySearch_MultipleActivities_MatchByTypeAndNotes()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 0, Notes = "running in pool" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "run", ActivityTypes, NoLocations).ToList();
        Assert.Equal(2, result.Count);
    }

    [Fact]
    public void FilterBySearch_ActivityWithNullNotes_DoesNotThrow_MatchesByAmount()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 15, Notes = null },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "15", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        result = ActivitySearchFilter.FilterBySearch(activities, "xyz", ActivityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_MissingActivityType_TypeNameTreatedAsEmpty_StillMatchesNotesOrAmount()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 99, When = DateTime.Now, Amount = 42, Notes = "custom" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "42", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
        result = ActivitySearchFilter.FilterBySearch(activities, "custom", ActivityTypes, NoLocations).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_EmptyActivitiesList_ReturnsEmpty()
    {
        var activities = new List<Activity>();
        var result = ActivitySearchFilter.FilterBySearch(activities, "run", ActivityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_EmptyActivitiesList_NullSearch_ReturnsEmpty()
    {
        var activities = new List<Activity>();
        var result = ActivitySearchFilter.FilterBySearch(activities, null, ActivityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_EmptyActivityTypesList_TypeNameEmpty_MatchesByNotesOrAmount()
    {
        var activityTypes = new List<ActivityType>();
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 10, Notes = "hello" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "hello", activityTypes, NoLocations).ToList();
        Assert.Single(result);
        result = ActivitySearchFilter.FilterBySearch(activities, "10", activityTypes, NoLocations).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_EmptyActivityTypesList_NoMatchOnNotesOrAmount_ReturnsEmpty()
    {
        var activityTypes = new List<ActivityType>();
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 10, Notes = "hello" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "Running", activityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }

    // FilterBySearch location-name matching tests

    [Fact]
    public void FilterBySearch_MatchByLocationName_IncludesMatchingActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, KnownLocationId = 10 },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 0, KnownLocationId = 20 },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "gym", ActivityTypes, KnownLocations).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterBySearch_LocationNameCaseInsensitive_Matches()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, KnownLocationId = 10 },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "GYM", ActivityTypes, KnownLocations).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_SearchTermNotInLocationName_ExcludesActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, KnownLocationId = 10, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "pool", ActivityTypes, KnownLocations).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_EmptyKnownLocationsList_LocationNameTreatedAsEmpty()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, KnownLocationId = 10, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "gym", ActivityTypes, NoLocations).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_NullKnownLocationId_LocationNameTreatedAsEmpty_NoMatch()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, KnownLocationId = null, Notes = "" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "gym", ActivityTypes, KnownLocations).ToList();
        Assert.Empty(result);
    }

    // FilterPrivate tests

    private static readonly List<ActivityType> ActivityTypesWithPrivate = new()
    {
        new() { Id = 1, Name = "Running", IsPrivate = false },
        new() { Id = 2, Name = "Meditation", IsPrivate = true },
        new() { Id = 3, Name = "Swimming", IsPrivate = false },
    };

    [Fact]
    public void FilterPrivate_NoSearch_HidesPrivateActivities()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 5 },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 10 },
        };
        var result = ActivitySearchFilter.FilterPrivate(activities, null, ActivityTypesWithPrivate).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterPrivate_EmptySearch_HidesPrivateActivities()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 5 },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 10 },
        };
        var result = ActivitySearchFilter.FilterPrivate(activities, "", ActivityTypesWithPrivate).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterPrivate_SearchMatchesPrivateTypeName_ShowsPrivateActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 5 },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 10 },
        };
        var result = ActivitySearchFilter.FilterPrivate(activities, "Medi", ActivityTypesWithPrivate).ToList();
        Assert.Equal(2, result.Count);
    }

    [Fact]
    public void FilterPrivate_SearchDoesNotMatchPrivateTypeName_HidesPrivateActivity()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 5 },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 10, Notes = "calm session" },
        };
        // "calm" matches notes but not the type name "Meditation" — private type stays hidden
        var result = ActivitySearchFilter.FilterPrivate(activities, "calm", ActivityTypesWithPrivate).ToList();
        Assert.Single(result);
        Assert.Equal(1, result[0].Id);
    }

    [Fact]
    public void FilterPrivate_SearchMatchesPrivateTypeName_CaseInsensitive()
    {
        var activities = new List<Activity>
        {
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 10 },
        };
        var result = ActivitySearchFilter.FilterPrivate(activities, "MEDITATION", ActivityTypesWithPrivate).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterPrivate_PublicActivitiesAlwaysPassThrough()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 5 },
            new() { Id = 3, ActivityTypeId = 3, When = DateTime.Now, Amount = 20 },
        };
        var result = ActivitySearchFilter.FilterPrivate(activities, null, ActivityTypesWithPrivate).ToList();
        Assert.Equal(2, result.Count);
    }

    [Fact]
    public void FilterPrivate_HomeChartCallPattern_ExcludesPrivateActivityTypes()
    {
        // Mirrors the call in Index.razor GetFilteredActivitiesAsync(): null search, no user input
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 5 },  // Running (public)
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 10 }, // Meditation (private)
            new() { Id = 3, ActivityTypeId = 3, When = DateTime.Now, Amount = 20 }, // Swimming (public)
        };
        var result = ActivitySearchFilter.FilterPrivate(activities, null, ActivityTypesWithPrivate).ToList();
        Assert.Equal(2, result.Count);
        Assert.DoesNotContain(result, a => a.ActivityTypeId == 2);
    }
}
