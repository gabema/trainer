namespace Trainer.Services;

using Trainer.Models;

internal class ActivityTypeService(IStorageService storageService) : IActivityTypeService
{
    private readonly IStorageService _storageService = storageService;
    private const string StorageKey = "activityTypes";
    private int _nextId = 1;

    private async Task<List<ActivityType>> GetAllUnsortedAsync()
    {
        var types = await _storageService.GetItemAsync<List<ActivityType>>(StorageKey).ConfigureAwait(false) ?? new List<ActivityType>();
        
        // Update next ID based on existing items
        if (types.Count > 0)
        {
            _nextId = types.Max(t => t.Id) + 1;
        }
        
        return types;
    }

    public async Task<List<ActivityType>> GetAllAsync()
    {
        var types = await GetAllUnsortedAsync().ConfigureAwait(false);
        return types.OrderBy(t => t.Name).ToList();
    }

    public async Task<ActivityType?> GetByIdAsync(int id)
    {
        var types = await GetAllAsync().ConfigureAwait(false);
        return types.FirstOrDefault(t => t.Id == id);
    }

    public async Task<ActivityType> AddAsync(ActivityType activityType)
    {
        ArgumentNullException.ThrowIfNull(activityType);
        var types = await GetAllUnsortedAsync().ConfigureAwait(false);
        activityType.Id = _nextId++;
        types.Add(activityType);
        await _storageService.SetItemAsync(StorageKey, types).ConfigureAwait(false);
        return activityType;
    }

    public async Task UpdateAsync(ActivityType activityType)
    {
        var types = await GetAllUnsortedAsync().ConfigureAwait(false);
        var index = types.FindIndex(t => t.Id == activityType.Id);
        if (index >= 0)
        {
            types[index] = activityType;
            await _storageService.SetItemAsync(StorageKey, types).ConfigureAwait(false);
        }
    }

    public async Task DeleteAsync(int id)
    {
        var types = await GetAllUnsortedAsync().ConfigureAwait(false);
        types.RemoveAll(t => t.Id == id);
        await _storageService.SetItemAsync(StorageKey, types).ConfigureAwait(false);
    }
}
