using Trainer.Models;
using Trainer.Services;

namespace Trainer.Tests.Services;

public class DateTimeHelperTests
{
    private readonly DateTime _now = new(2025, 1, 15, 14, 30, 0);

    [Theory]
    [InlineData(0, 15, "1 minute ago")]   // 15 seconds ago
    [InlineData(1, 0, "1 minute ago")]    // 1 minute ago
    [InlineData(15, 0, "15 minutes ago")] // 15 minutes ago
    [InlineData(119, 0, "119 minutes ago")] // 119 minutes ago
    [InlineData(119, 59, "119 minutes ago")] // 119m 59s ago
    public void FormatWhenDateTime_RecentTimes_ReturnsMinutesAgo(int minutesAgo, int secondsAgo, string expected)
    {
        var when = _now.AddMinutes(-minutesAgo).AddSeconds(-secondsAgo);
        var result = DateTimeHelper.FormatWhenDateTime(when, _now);
        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData(12, 30, "12:30 pm")] // Exactly 2 hours ago (14:30 - 2h = 12:30)
    [InlineData(11, 30, "11:30 am")] // 3 hours ago
    [InlineData(2, 25, "2:25 am")]   // Early morning same day
    [InlineData(0, 15, "12:15 am")]  // Midnight + 15m same day
    public void FormatWhenDateTime_SameDayOlderThan2Hours_ReturnsTimeOnly(int hour, int minute, string expected)
    {
        var when = new DateTime(_now.Year, _now.Month, _now.Day, hour, minute, 0);
        var result = DateTimeHelper.FormatWhenDateTime(when, _now);
        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData(8, 30, "yesterday @ 8:30 am")]
    [InlineData(15, 42, "yesterday @ 3:42 pm")]
    [InlineData(2, 25, "yesterday @ 2:25 am")]
    [InlineData(0, 0, "yesterday @ 12:00 am")]
    public void FormatWhenDateTime_Yesterday_ReturnsYesterdayAtTime(int hour, int minute, string expected)
    {
        var yesterday = _now.AddDays(-1);
        var when = new DateTime(yesterday.Year, yesterday.Month, yesterday.Day, hour, minute, 0);
        var result = DateTimeHelper.FormatWhenDateTime(when, _now);
        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData(2025, 1, 13, 10, 22, "Jan 13 @ 10:22 am")] // Two days ago
    [InlineData(2025, 1, 8, 9, 15, "Jan 8 @ 9:15 am")]     // One week ago
    [InlineData(2024, 12, 15, 16, 45, "Dec 15 @ 4:45 pm")] // One month ago
    [InlineData(2024, 1, 10, 10, 22, "Jan 10 @ 10:22 am")] // One year ago
    [InlineData(2025, 1, 20, 10, 0, "Jan 20 @ 10:00 am")]  // Future date
    public void FormatWhenDateTime_OtherDates_ReturnsShortDateAndTime(int year, int month, int day, int hour, int minute, string expected)
    {
        var when = new DateTime(year, month, day, hour, minute, 0);
        var result = DateTimeHelper.FormatWhenDateTime(when, _now);
        Assert.Equal(expected, result);
    }

    [Fact]
    public void FormatWhenDateTime_NoNowParameter_UsesCurrentTime()
    {
        var when = DateTime.Now.AddMinutes(-30);
        var result = DateTimeHelper.FormatWhenDateTime(when);
        Assert.Contains("minutes ago", result);
    }

    [Theory]
    [InlineData(DurationOption.Last24Hours, 1)]
    [InlineData(DurationOption.Last7Days, 7)]
    [InlineData(DurationOption.Last4Weeks, 28)]
    public void GetDateRange_RelativeDurations_ReturnsCorrectRange(DurationOption duration, int days)
    {
        var (start, end) = DateTimeHelper.GetDateRange(duration, _now);
        Assert.Equal(_now.AddDays(-days), start);
        Assert.Equal(_now, end);
    }

    [Fact]
    public void GetDateRange_Week_ReturnsCurrentWeekFromMonday()
    {
        // Jan 15 2025 is a Wednesday
        var (start, end) = DateTimeHelper.GetDateRange(DurationOption.Week, _now);
        
        var expectedStart = new DateTime(2025, 1, 13); // Monday
        var expectedEnd = new DateTime(2025, 1, 19, 23, 59, 59); // Sunday

        Assert.Equal(expectedStart, start);
        Assert.Equal(expectedEnd, end);
    }
}
