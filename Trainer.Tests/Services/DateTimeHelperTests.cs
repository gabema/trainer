using Trainer.Services;

namespace Trainer.Tests.Services;

public class DateTimeHelperTests
{
    [Fact]
    public void FormatWhenDateTime_LessThan1MinuteAgo_Returns1MinuteAgo()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 14, 29, 45); // 15 seconds ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("1 minute ago", result);
    }

    [Fact]
    public void FormatWhenDateTime_1MinuteAgo_Returns1MinuteAgo()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 14, 29, 0); // 1 minute ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("1 minute ago", result);
    }

    [Fact]
    public void FormatWhenDateTime_15MinutesAgo_Returns15MinutesAgo()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 14, 15, 0); // 15 minutes ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("15 minutes ago", result);
    }

    [Fact]
    public void FormatWhenDateTime_119MinutesAgo_Returns119MinutesAgo()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 12, 31, 0); // 119 minutes ago (just under 2 hours)

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("119 minutes ago", result);
    }

    [Fact]
    public void FormatWhenDateTime_2HoursAgoSameDay_ReturnsTimeOnly()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 12, 30, 0); // Exactly 2 hours ago, same day

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("12:30 pm", result);
    }

    [Fact]
    public void FormatWhenDateTime_3HoursAgoSameDay_ReturnsTimeOnly()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 11, 30, 0); // 3 hours ago, same day

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("11:30 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_EarlyMorningSameDay_ReturnsTimeOnly()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 2, 25, 0); // 12 hours ago, same day, early morning

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("2:25 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_LastTimeOfSameDay_ReturnsTimeOnly()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 0, 15, 0); // Early morning same day

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("12:15 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_YesterdayAtMorning_ReturnsYesterdayAtTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 14, 8, 30, 0); // Yesterday morning

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("yesterday @ 8:30 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_YesterdayAtAfternoon_ReturnsYesterdayAtTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 14, 15, 42, 0); // Yesterday afternoon

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("yesterday @ 3:42 pm", result);
    }

    [Fact]
    public void FormatWhenDateTime_YesterdayAtEarlyMorning_ReturnsYesterdayAtTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 14, 2, 25, 0); // Yesterday early morning

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("yesterday @ 2:25 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_YesterdayAtMidnight_ReturnsYesterdayAtTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 14, 0, 0, 0); // Yesterday at midnight

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("yesterday @ 12:00 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_TwoDaysAgo_ReturnsShortDateAndTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 13, 10, 22, 0); // Two days ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("Jan 13 @ 10:22 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_OneWeekAgo_ReturnsShortDateAndTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 8, 9, 15, 0); // One week ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("Jan 8 @ 9:15 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_OneMonthAgo_ReturnsShortDateAndTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2024, 12, 15, 16, 45, 0); // One month ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("Dec 15 @ 4:45 pm", result);
    }

    [Fact]
    public void FormatWhenDateTime_OneYearAgo_ReturnsShortDateAndTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2024, 1, 10, 10, 22, 0); // One year ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("Jan 10 @ 10:22 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_FutureDate_ReturnsShortDateAndTime()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 20, 10, 0, 0); // Future date (edge case)

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("Jan 20 @ 10:00 am", result);
    }

    [Fact]
    public void FormatWhenDateTime_ExactlyAt2HoursBoundary_ReturnsTimeOnly()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 12, 30, 0); // Exactly 2 hours ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        // Should show time only (not "120 minutes ago")
        Assert.Equal("12:30 pm", result);
    }

    [Fact]
    public void FormatWhenDateTime_119Minutes59SecondsAgo_Returns119MinutesAgo()
    {
        // Arrange
        var now = new DateTime(2025, 1, 15, 14, 30, 0);
        var when = new DateTime(2025, 1, 15, 12, 30, 1); // 119 minutes 59 seconds ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when, now);

        // Assert
        Assert.Equal("119 minutes ago", result);
    }

    [Fact]
    public void FormatWhenDateTime_NoNowParameter_UsesCurrentTime()
    {
        // Arrange
        var when = DateTime.Now.AddMinutes(-30); // 30 minutes ago

        // Act
        var result = DateTimeHelper.FormatWhenDateTime(when);

        // Assert
        // Should use DateTime.Now (current time) as the reference
        Assert.Contains("minutes ago", result);
    }
}

