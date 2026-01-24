using Trainer.Models;
using Trainer.Services;

namespace Trainer.Tests.Services;

public class GoalServiceTests
{
    private readonly GoalService _service;

    public GoalServiceTests()
    {
        _service = new GoalService();
    }

    [Theory]
    [InlineData(DurationOption.Last24Hours, 10, 50, 10)]
    [InlineData(DurationOption.Last7Days, 10, 50, 50)]
    [InlineData(DurationOption.Week, 10, 50, 50)]
    public void GetGoalAmount_StandardDurations_ReturnsCorrectAmount(DurationOption duration, int daily, int weekly, int expected)
    {
        var type = new ActivityType { DailyAmount = daily, WeeklyAmount = weekly };
        var result = _service.GetGoalAmount(type, duration);
        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData(null, 50, 200)] // Weekly only: 50 * 4
    [InlineData(10, null, 280)] // Daily only: 10 * 28
    [InlineData(10, 50, 200)]   // Both: 50 * 4 (prefers weekly)
    [InlineData(null, null, null)] // Neither
    public void GetGoalAmount_Last4Weeks_ReturnsCorrectAmount(int? daily, int? weekly, int? expected)
    {
        var type = new ActivityType { DailyAmount = daily, WeeklyAmount = weekly };
        var result = _service.GetGoalAmount(type, DurationOption.Last4Weeks);
        Assert.Equal(expected, result);
    }
}
