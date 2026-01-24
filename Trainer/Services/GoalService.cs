namespace Trainer.Services;

using Trainer.Models;

public class GoalService : IGoalService
{
    public int? GetGoalAmount(ActivityType activityType, DurationOption duration)
    {
        return duration switch
        {
            DurationOption.Last24Hours => activityType.DailyAmount,
            DurationOption.Last7Days => activityType.WeeklyAmount,
            DurationOption.Week => activityType.WeeklyAmount,
            DurationOption.Last4Weeks => GetLast4WeeksGoal(activityType),
            _ => null
        };
    }

    private int? GetLast4WeeksGoal(ActivityType activityType)
    {
        if (activityType.WeeklyAmount.HasValue)
        {
            return activityType.WeeklyAmount.Value * 4;
        }
        
        if (activityType.DailyAmount.HasValue)
        {
            return activityType.DailyAmount.Value * 28;
        }

        return null;
    }
}
