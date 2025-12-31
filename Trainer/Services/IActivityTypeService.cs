using Trainer.Models;

namespace Trainer.Services;

public interface IActivityTypeService
{
    Task<List<ActivityType>> GetAllAsync();
    Task<ActivityType?> GetByIdAsync(int id);
    Task<ActivityType> AddAsync(ActivityType activityType);
    Task UpdateAsync(ActivityType activityType);
    Task DeleteAsync(int id);
}

