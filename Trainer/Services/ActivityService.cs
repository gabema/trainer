using Trainer.Models;

namespace Trainer.Services;

public class ActivityService : IActivityService
{
    private readonly IndexedDbStorageService _storageService;
    private const string StorageKey = "activities";
    private int _nextId = 1;
    private bool _nextIdInitialized = false;

    public ActivityService(IStorageService storageService)
    {
        // Cast to IndexedDbStorageService for weekly operations
        if (storageService is not IndexedDbStorageService indexedDbService)
        {
            throw new InvalidOperationException("ActivityService requires IndexedDbStorageService");
        }
        _storageService = indexedDbService;
    }

    private async Task EnsureNextIdInitializedAsync()
    {
        if (_nextIdInitialized)
            return;

        var activities = await GetAllAsync();
        if (activities.Any())
        {
            _nextId = activities.Max(a => a.Id) + 1;
        }
        _nextIdInitialized = true;
    }

    public async Task<List<Activity>> GetAllAsync(DateTime? startDate = null, DateTime? endDate = null)
    {
        List<Activity> activities;

        if (startDate.HasValue || endDate.HasValue)
        {
            // Load only activities in the specified date range
            var start = startDate ?? DateTime.MinValue;
            var end = endDate ?? DateTime.MaxValue;
            var weekKeys = WeekHelper.GetWeekKeysInRange(start, end).ToList();
            
            activities = await _storageService.GetActivitiesByWeekRangeAsync(weekKeys);
        }
        else
        {
            // Load all activities
            activities = await _storageService.GetItemAsync<List<Activity>>(StorageKey) ?? new List<Activity>();
        }

        // Update next ID based on existing items (always check all activities, not just filtered results)
        if (!_nextIdInitialized)
        {
            // Need to check all activities to ensure we have the max ID, regardless of the filtered result
            var allActivities = await _storageService.GetItemAsync<List<Activity>>(StorageKey) ?? new List<Activity>();
            if (allActivities.Any())
            {
                _nextId = allActivities.Max(a => a.Id) + 1;
            }
            _nextIdInitialized = true;
        }

        return activities.OrderByDescending(a => a.When).ToList();
    }

    public async Task<Activity?> GetByIdAsync(int id)
    {
        // For getting by ID, we need to search across all weeks
        // For efficiency, we could cache this or search more intelligently
        var activities = await GetAllAsync();
        return activities.FirstOrDefault(a => a.Id == id);
    }

    public async Task<Activity> AddAsync(Activity activity)
    {
        await EnsureNextIdInitializedAsync();
        activity.Id = _nextId++;

        // Get the week key for this activity
        var weekKey = WeekHelper.GetWeekKey(activity.When);

        // Get existing activities for this week
        var weekActivities = await _storageService.GetActivitiesByWeekAsync(weekKey);
        weekActivities.Add(activity);

        // Save the week's activities
        await _storageService.SetActivitiesForWeekAsync(weekKey, weekActivities);

        _nextIdInitialized = true;
        return activity;
    }

    public async Task UpdateAsync(Activity activity)
    {
        // Find the existing activity to determine its current week
        var existingActivity = await GetByIdAsync(activity.Id);
        if (existingActivity == null)
            return;

        var oldWeekKey = WeekHelper.GetWeekKey(existingActivity.When);
        var newWeekKey = WeekHelper.GetWeekKey(activity.When);

        if (oldWeekKey == newWeekKey)
        {
            // Activity stays in the same week
            var weekActivities = await _storageService.GetActivitiesByWeekAsync(oldWeekKey);
            // Remove any existing entries with this ID first to prevent duplicates
            weekActivities.RemoveAll(a => a.Id == activity.Id);
            // Add the updated activity
            weekActivities.Add(activity);
            await _storageService.SetActivitiesForWeekAsync(oldWeekKey, weekActivities);
        }
        else
        {
            // Activity moved to a different week - remove from old week, add to new week
            var oldWeekActivities = await _storageService.GetActivitiesByWeekAsync(oldWeekKey);
            oldWeekActivities.RemoveAll(a => a.Id == activity.Id);
            
            // If the old week is now empty, delete the week key; otherwise save the updated list
            if (oldWeekActivities.Count == 0)
            {
                await _storageService.RemoveActivitiesForWeekAsync(oldWeekKey);
            }
            else
            {
                await _storageService.SetActivitiesForWeekAsync(oldWeekKey, oldWeekActivities);
            }

            var newWeekActivities = await _storageService.GetActivitiesByWeekAsync(newWeekKey);
            // Remove any existing entries with this ID first to prevent duplicates
            newWeekActivities.RemoveAll(a => a.Id == activity.Id);
            // Add the updated activity
            newWeekActivities.Add(activity);
            await _storageService.SetActivitiesForWeekAsync(newWeekKey, newWeekActivities);
        }
    }

    public async Task DeleteAsync(int id)
    {
        // Find the activity to determine which week it's in
        var activity = await GetByIdAsync(id);
        if (activity == null)
            return;

        var weekKey = WeekHelper.GetWeekKey(activity.When);
        var weekActivities = await _storageService.GetActivitiesByWeekAsync(weekKey);
        weekActivities.RemoveAll(a => a.Id == id);
        await _storageService.SetActivitiesForWeekAsync(weekKey, weekActivities);
    }

    public async Task<List<Activity>> GetByActivityTypeIdAsync(int activityTypeId)
    {
        var activities = await GetAllAsync();
        return activities.Where(a => a.ActivityTypeId == activityTypeId).ToList();
    }

    public async Task<List<string>> GetAllAvailableWeekKeysAsync()
    {
        return await _storageService.GetAllAvailableWeekKeysAsync();
    }
}

