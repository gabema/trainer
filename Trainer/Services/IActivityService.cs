namespace Trainer.Services;

using Trainer.Models;

public interface IActivityService
{
    Task<List<Activity>> GetAllAsync(DateTime? startDate = null, DateTime? endDate = null);
    Task<Activity?> GetByIdAsync(int id);
    Task<Activity> AddAsync(Activity activity);
    Task UpdateAsync(Activity activity);
    Task DeleteAsync(int id);
    Task<List<Activity>> GetByActivityTypeIdAsync(int activityTypeId);
    Task<List<string>> GetAllAvailableWeekKeysAsync();
    Task RecalculateNextIdAsync();
}
