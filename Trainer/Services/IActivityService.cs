using Trainer.Models;

namespace Trainer.Services;

public interface IActivityService
{
    Task<List<Activity>> GetAllAsync();
    Task<Activity?> GetByIdAsync(int id);
    Task<Activity> AddAsync(Activity activity);
    Task UpdateAsync(Activity activity);
    Task DeleteAsync(int id);
    Task<List<Activity>> GetByActivityTypeIdAsync(int activityTypeId);
}

