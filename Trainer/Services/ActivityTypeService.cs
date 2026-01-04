using Trainer.Models;

namespace Trainer.Services;

public class ActivityTypeService : IActivityTypeService
{
    private readonly IStorageService _storageService;
    private const string StorageKey = "activityTypes";
    private int _nextId = 1;

    public ActivityTypeService(IStorageService storageService)
    {
        _storageService = storageService;
    }

    public async Task<List<ActivityType>> GetAllAsync()
    {
        var types = await _storageService.GetItemAsync<List<ActivityType>>(StorageKey) ?? new List<ActivityType>();
        
        // Update next ID based on existing items
        if (types.Any())
        {
            _nextId = types.Max(t => t.Id) + 1;
        }
        
        return types.OrderBy(t => t.Name).ToList();
    }

    public async Task<ActivityType?> GetByIdAsync(int id)
    {
        var types = await GetAllAsync();
        return types.FirstOrDefault(t => t.Id == id);
    }

    public async Task<ActivityType> AddAsync(ActivityType activityType)
    {
        var types = await GetAllAsync();
        activityType.Id = _nextId++;
        types.Add(activityType);
        await _storageService.SetItemAsync(StorageKey, types);
        return activityType;
    }

    public async Task UpdateAsync(ActivityType activityType)
    {
        var types = await GetAllAsync();
        var index = types.FindIndex(t => t.Id == activityType.Id);
        if (index >= 0)
        {
            types[index] = activityType;
            await _storageService.SetItemAsync(StorageKey, types);
        }
    }

    public async Task DeleteAsync(int id)
    {
        var types = await GetAllAsync();
        types.RemoveAll(t => t.Id == id);
        await _storageService.SetItemAsync(StorageKey, types);
    }
}

