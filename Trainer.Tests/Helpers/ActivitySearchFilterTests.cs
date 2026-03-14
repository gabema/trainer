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

    [Fact]
    public void FilterBySearch_NullSearch_ReturnsInputUnchanged()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 10, Notes = "run" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, null, ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, searchTerm, ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "Run", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "morning", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "15", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "15", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, searchTerm, ActivityTypes).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_CaseInsensitiveNotes_MatchesRegardlessOfCase()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "my NOTE here" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "note", ActivityTypes).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_NoMatches_ReturnsEmpty()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 0, Notes = "run" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "xyz", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "read", ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "run", ActivityTypes).ToList();
        Assert.Equal(2, result.Count);
    }

    [Fact]
    public void FilterBySearch_MissingActivityType_TypeNameTreatedAsEmpty_StillMatchesNotesOrAmount()
    {
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 99, When = DateTime.Now, Amount = 42, Notes = "custom" },
        };
        var result = ActivitySearchFilter.FilterBySearch(activities, "42", ActivityTypes).ToList();
        Assert.Single(result);
        result = ActivitySearchFilter.FilterBySearch(activities, "custom", ActivityTypes).ToList();
        Assert.Single(result);
    }

    [Fact]
    public void FilterBySearch_EmptyActivitiesList_ReturnsEmpty()
    {
        var activities = new List<Activity>();
        var result = ActivitySearchFilter.FilterBySearch(activities, "run", ActivityTypes).ToList();
        Assert.Empty(result);
    }

    [Fact]
    public void FilterBySearch_EmptyActivitiesList_NullSearch_ReturnsEmpty()
    {
        var activities = new List<Activity>();
        var result = ActivitySearchFilter.FilterBySearch(activities, null, ActivityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "hello", activityTypes).ToList();
        Assert.Single(result);
        result = ActivitySearchFilter.FilterBySearch(activities, "10", activityTypes).ToList();
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
        var result = ActivitySearchFilter.FilterBySearch(activities, "Running", activityTypes).ToList();
        Assert.Empty(result);
    }
}
