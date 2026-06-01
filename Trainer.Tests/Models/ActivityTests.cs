namespace Trainer.Tests.Models;

using Trainer.Models;

public class ActivityTests
{
    [Fact]
    public void Activity_DefaultKnownLocationId_IsNull()
    {
        var activity = new Activity { When = DateTime.Now, Amount = 10 };

        Assert.Null(activity.KnownLocationId);
    }

    [Fact]
    public void Activity_WithKnownLocationId_StoresValue()
    {
        var activity = new Activity
        {
            When = DateTime.Now,
            Amount = 10,
            KnownLocationId = 42
        };

        Assert.Equal(42, activity.KnownLocationId);
    }

    [Fact]
    public void Activity_ClearKnownLocationId_BecomesNull()
    {
        var activity = new Activity
        {
            When = DateTime.Now,
            Amount = 10,
            KnownLocationId = 42
        };

        activity.KnownLocationId = null;

        Assert.Null(activity.KnownLocationId);
    }

    [Fact]
    public void DuplicateFrom_WithKnownLocationId_CopiesId()
    {
        var source = new Activity
        {
            Id = 1,
            ActivityTypeId = 3,
            When = DateTime.Now.AddDays(-1),
            Amount = 5000,
            Notes = "morning run",
            DurationSeconds = 1800,
            KnownLocationId = 99
        };

        var duplicate = new Activity
        {
            Id = 0,
            ActivityTypeId = source.ActivityTypeId,
            When = DateTime.Now,
            Amount = source.Amount,
            Notes = source.Notes,
            DurationSeconds = source.DurationSeconds,
            KnownLocationId = source.KnownLocationId
        };

        Assert.Equal(source.KnownLocationId, duplicate.KnownLocationId);
        Assert.Equal(0, duplicate.Id);
    }

    [Fact]
    public void DuplicateFrom_WithNullKnownLocationId_CopiesNull()
    {
        var source = new Activity
        {
            Id = 1,
            ActivityTypeId = 3,
            When = DateTime.Now.AddDays(-1),
            Amount = 5000,
            KnownLocationId = null
        };

        var duplicate = new Activity
        {
            Id = 0,
            ActivityTypeId = source.ActivityTypeId,
            When = DateTime.Now,
            Amount = source.Amount,
            KnownLocationId = source.KnownLocationId
        };

        Assert.Null(duplicate.KnownLocationId);
    }
}
