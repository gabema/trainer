namespace Trainer.Services;

using Trainer.Models;

public interface IGoalService
{
    int? GetGoalAmount(ActivityType activityType, DurationOption duration);
}
