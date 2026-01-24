using Trainer.Models;

namespace Trainer.Services;

public interface IGoalService
{
    int? GetGoalAmount(ActivityType activityType, DurationOption duration);
}
