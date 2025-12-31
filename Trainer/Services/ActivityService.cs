using Trainer.Models;

namespace Trainer.Services;

public class ActivityService : IActivityService
{
    private readonly IStorageService _storageService;
    private const string StorageKey = "activities";
    private int _nextId = 1;

    public ActivityService(IStorageService storageService)
    {
        _storageService = storageService;
    }

    public async Task<List<Activity>> GetAllAsync()
    {
        var activities = await _storageService.GetItemAsync<List<Activity>>(StorageKey) ?? new List<Activity>();
        
        // Update next ID based on existing items
        if (activities.Any())
        {
            _nextId = activities.Max(a => a.Id) + 1;
        }
        
        return activities;
    }

    public async Task<Activity?> GetByIdAsync(int id)
    {
        var activities = await GetAllAsync();
        return activities.FirstOrDefault(a => a.Id == id);
    }

    public async Task<Activity> AddAsync(Activity activity)
    {
        var activities = await GetAllAsync();
        activity.Id = _nextId++;
        activities.Add(activity);
        await _storageService.SetItemAsync(StorageKey, activities);
        return activity;
    }

    public async Task UpdateAsync(Activity activity)
    {
        var activities = await GetAllAsync();
        var index = activities.FindIndex(a => a.Id == activity.Id);
        if (index >= 0)
        {
            activities[index] = activity;
            await _storageService.SetItemAsync(StorageKey, activities);
        }
    }

    public async Task DeleteAsync(int id)
    {
        var activities = await GetAllAsync();
        activities.RemoveAll(a => a.Id == id);
        await _storageService.SetItemAsync(StorageKey, activities);
    }

    public async Task<List<Activity>> GetByActivityTypeIdAsync(int activityTypeId)
    {
        var activities = await GetAllAsync();
        return activities.Where(a => a.ActivityTypeId == activityTypeId).ToList();
    }
}

