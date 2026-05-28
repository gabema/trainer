namespace Trainer.Tests.Models;

using Trainer.Models;

public class ActivityTests
{
    [Fact]
    public void Activity_DefaultCoordinates_AreNull()
    {
        var activity = new Activity { When = DateTime.Now, Amount = 10 };

        Assert.Null(activity.Latitude);
        Assert.Null(activity.Longitude);
    }

    [Fact]
    public void Activity_WithCoordinates_StoresValues()
    {
        var activity = new Activity
        {
            When = DateTime.Now,
            Amount = 10,
            Latitude = 37.77493,
            Longitude = -122.41942
        };

        Assert.Equal(37.77493, activity.Latitude);
        Assert.Equal(-122.41942, activity.Longitude);
    }

    [Fact]
    public void Activity_ClearCoordinates_BecomeNull()
    {
        var activity = new Activity
        {
            When = DateTime.Now,
            Amount = 10,
            Latitude = 37.77493,
            Longitude = -122.41942
        };

        activity.Latitude = null;
        activity.Longitude = null;

        Assert.Null(activity.Latitude);
        Assert.Null(activity.Longitude);
    }

    [Fact]
    public void DuplicateFrom_WithCoordinates_CopiesCoordinates()
    {
        var source = new Activity
        {
            Id = 1,
            ActivityTypeId = 3,
            When = DateTime.Now.AddDays(-1),
            Amount = 5000,
            Notes = "morning run",
            DurationSeconds = 1800,
            Latitude = 37.77493,
            Longitude = -122.41942
        };

        var duplicate = new Activity
        {
            Id = 0,
            ActivityTypeId = source.ActivityTypeId,
            When = DateTime.Now,
            Amount = source.Amount,
            Notes = source.Notes,
            DurationSeconds = source.DurationSeconds,
            Latitude = source.Latitude,
            Longitude = source.Longitude
        };

        Assert.Equal(source.Latitude, duplicate.Latitude);
        Assert.Equal(source.Longitude, duplicate.Longitude);
        Assert.Equal(0, duplicate.Id);
    }

    [Fact]
    public void DuplicateFrom_WithNullCoordinates_CopiesNulls()
    {
        var source = new Activity
        {
            Id = 1,
            ActivityTypeId = 3,
            When = DateTime.Now.AddDays(-1),
            Amount = 5000,
            Latitude = null,
            Longitude = null
        };

        var duplicate = new Activity
        {
            Id = 0,
            ActivityTypeId = source.ActivityTypeId,
            When = DateTime.Now,
            Amount = source.Amount,
            Latitude = source.Latitude,
            Longitude = source.Longitude
        };

        Assert.Null(duplicate.Latitude);
        Assert.Null(duplicate.Longitude);
    }
}
