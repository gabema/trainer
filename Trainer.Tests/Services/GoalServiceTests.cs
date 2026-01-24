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

    [Fact]
    public void GetGoalAmount_Last24Hours_UsesDailyAmount()
    {
        var type = new ActivityType { DailyAmount = 10, WeeklyAmount = 50 };
        var result = _service.GetGoalAmount(type, DurationOption.Last24Hours);
        Assert.Equal(10, result);
    }

    [Fact]
    public void GetGoalAmount_Last7Days_UsesWeeklyAmount()
    {
        var type = new ActivityType { DailyAmount = 10, WeeklyAmount = 50 };
        var result = _service.GetGoalAmount(type, DurationOption.Last7Days);
        Assert.Equal(50, result);
    }

    [Fact]
    public void GetGoalAmount_Week_UsesWeeklyAmount()
    {
        var type = new ActivityType { DailyAmount = 10, WeeklyAmount = 50 };
        var result = _service.GetGoalAmount(type, DurationOption.Week);
        Assert.Equal(50, result);
    }

    [Fact]
    public void GetGoalAmount_Last4Weeks_WithWeeklyAmount_ReturnsWeeklyTimes4()
    {
        var type = new ActivityType { WeeklyAmount = 50 };
        var result = _service.GetGoalAmount(type, DurationOption.Last4Weeks);
        Assert.Equal(200, result); // 50 * 4
    }

    [Fact]
    public void GetGoalAmount_Last4Weeks_WithDailyAmountOnly_ReturnsDailyTimes28()
    {
        var type = new ActivityType { DailyAmount = 10 };
        var result = _service.GetGoalAmount(type, DurationOption.Last4Weeks);
        Assert.Equal(280, result); // 10 * 28
    }

    [Fact]
    public void GetGoalAmount_Last4Weeks_WithBothAmounts_PrefersWeeklyTimes4()
    {
        var type = new ActivityType { DailyAmount = 10, WeeklyAmount = 50 };
        var result = _service.GetGoalAmount(type, DurationOption.Last4Weeks);
        Assert.Equal(200, result); // 50 * 4, ignoring 10 * 28 = 280
    }

    [Fact]
    public void GetGoalAmount_Last4Weeks_WithNoAmounts_ReturnsNull()
    {
        var type = new ActivityType();
        var result = _service.GetGoalAmount(type, DurationOption.Last4Weeks);
        Assert.Null(result);
    }
}
