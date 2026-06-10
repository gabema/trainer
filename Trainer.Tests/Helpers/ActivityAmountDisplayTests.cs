namespace Trainer.Tests.Helpers;

using Trainer.Helpers;
using Trainer.Models;

public class ActivityAmountDisplayTests
{
    private static readonly ActivityType RunType = new() { Id = 1, Name = "Running", Unit = "km" };
    private static readonly ActivityType NoUnitType = new() { Id = 2, Name = "Pushups" };
    private static readonly KnownLocation GymLocation = new() { Id = 10, Name = "Gym", Latitude = 0, Longitude = 0 };

    private static Activity Make(int activityTypeId = 1, int amount = 5, int? durationSeconds = null, int? knownLocationId = null) =>
        new() { Id = 1, ActivityTypeId = activityTypeId, Amount = amount, DurationSeconds = durationSeconds, KnownLocationId = knownLocationId, When = DateTime.Now };

    // ── Amount formatting ────────────────────────────────────────────────────

    [Fact]
    public void Format_AmountOnly_NoUnitNoDurationNoLocation()
    {
        var result = ActivityAmountDisplay.Format(Make(2, 10), [NoUnitType], []);
        Assert.Equal("10", result);
    }

    [Fact]
    public void Format_AmountWithUnit()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5), [RunType], []);
        Assert.Equal("5 km", result);
    }

    // ── Duration formatting ──────────────────────────────────────────────────

    [Fact]
    public void Format_AmountWithMinutesOnlyDuration()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5, durationSeconds: 1200), [RunType], []);
        Assert.Equal("5 km for 20m", result);
    }

    [Fact]
    public void Format_AmountWithMinutesAndSecondsDuration()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5, durationSeconds: 330), [RunType], []);
        Assert.Equal("5 km for 5m 30s", result);
    }

    [Fact]
    public void Format_AmountWithUnitAndDuration()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 10, durationSeconds: 600), [RunType], []);
        Assert.Equal("10 km for 10m", result);
    }

    // ── Location formatting ──────────────────────────────────────────────────

    [Fact]
    public void Format_AmountWithUnitDurationAndLocation()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5, durationSeconds: 1800, knownLocationId: 10), [RunType], [GymLocation]);
        Assert.Equal("5 km for 30m @ Gym", result);
    }

    [Fact]
    public void Format_AmountWithUnitAndLocationNoDuration()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5, knownLocationId: 10), [RunType], [GymLocation]);
        Assert.Equal("5 km @ Gym", result);
    }

    [Fact]
    public void Format_KnownLocationIdSetButNoMatchInList_OmitsLocationSuffix()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5, knownLocationId: 99), [RunType], [GymLocation]);
        Assert.Equal("5 km", result);
    }

    [Fact]
    public void Format_KnownLocationsEmpty_OmitsLocationSuffix()
    {
        var result = ActivityAmountDisplay.Format(Make(1, 5, knownLocationId: 10), [RunType], []);
        Assert.Equal("5 km", result);
    }

    // ── FormatDuration edge cases ────────────────────────────────────────────

    [Fact]
    public void FormatDuration_SecondsOnly()
    {
        Assert.Equal("45s", ActivityAmountDisplay.FormatDuration(45));
    }

    [Theory]
    [InlineData(305, "5m 5s")]   // single-digit seconds are not zero-padded
    [InlineData(330, "5m 30s")]  // two-digit seconds shown as-is
    [InlineData(600, "10m")]     // whole minutes omit the seconds component
    public void FormatDuration_MinutesAndSeconds_NoLeadingZeroOnSeconds(int durationSeconds, string expected)
    {
        Assert.Equal(expected, ActivityAmountDisplay.FormatDuration(durationSeconds));
    }

    [Fact]
    public void FormatDuration_NullOrZero_ReturnsNull()
    {
        Assert.Null(ActivityAmountDisplay.FormatDuration(null));
        Assert.Null(ActivityAmountDisplay.FormatDuration(0));
    }
}
