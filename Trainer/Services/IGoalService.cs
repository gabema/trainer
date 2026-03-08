namespace Trainer.Services;

using Trainer.Models;

internal interface IGoalService
{
    int? GetGoalAmount(ActivityType activityType, DurationOption duration);
}
