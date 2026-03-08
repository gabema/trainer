namespace Trainer.Services;

using Trainer.Models;

internal class ActivityService(IStorageService storageService) : IActivityService
{
    private readonly IndexedDbStorageService _storageService = storageService as IndexedDbStorageService ?? throw new InvalidOperationException("ActivityService requires IndexedDbStorageService");
    private const string StorageKey = "activities";
    private const string NextIdKey = "activityNextId";
    private int _nextId = 1;
    private bool _nextIdInitialized;

    private async Task EnsureNextIdInitializedAsync()
    {
        if (_nextIdInitialized)
            return;

        // Try to load from localStorage first
        var storedNextId = await _storageService.GetItemAsync<int?>(NextIdKey).ConfigureAwait(false);
        if (storedNextId.HasValue && storedNextId.Value > 0)
        {
            _nextId = storedNextId.Value;
        }
        else
        {
            // Fallback: calculate from existing activities (for migration)
            var activities = await _storageService.GetItemAsync<List<Activity>>(StorageKey).ConfigureAwait(false) ?? new List<Activity>();
            if (activities.Count > 0)
            {
                _nextId = activities.Max(a => a.Id) + 1;
                // Persist the calculated value
                await _storageService.SetItemAsync(NextIdKey, _nextId).ConfigureAwait(false);
            }
            else
            {
                // No activities, start at 1
                _nextId = 1;
                await _storageService.SetItemAsync(NextIdKey, _nextId).ConfigureAwait(false);
            }
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
            
            activities = await _storageService.GetActivitiesByWeekRangeAsync(weekKeys).ConfigureAwait(false);
        }
        else
        {
            // Load all activities
            activities = await _storageService.GetItemAsync<List<Activity>>(StorageKey).ConfigureAwait(false) ?? new List<Activity>();
        }

        // Initialize nextId if not already done
        if (!_nextIdInitialized)
        {
            await EnsureNextIdInitializedAsync().ConfigureAwait(false);
        }

        return activities.OrderByDescending(a => a.When).ToList();
    }

    public async Task<Activity?> GetByIdAsync(int id)
    {
        // For getting by ID, we need to search across all weeks
        // For efficiency, we could cache this or search more intelligently
        var activities = await GetAllAsync().ConfigureAwait(false);
        return activities.FirstOrDefault(a => a.Id == id);
    }

    public async Task<Activity> AddAsync(Activity activity)
    {
        ArgumentNullException.ThrowIfNull(activity);
        await EnsureNextIdInitializedAsync().ConfigureAwait(false);
        activity.Id = _nextId++;
        
        // Persist the updated nextId to localStorage
        await _storageService.SetItemAsync(NextIdKey, _nextId).ConfigureAwait(false);

        // Get the week key for this activity
        var weekKey = WeekHelper.GetWeekKey(activity.When);

        // Get existing activities for this week
        var weekActivities = await _storageService.GetActivitiesByWeekAsync(weekKey).ConfigureAwait(false);
        weekActivities.Add(activity);

        // Save the week's activities
        await _storageService.SetActivitiesForWeekAsync(weekKey, weekActivities).ConfigureAwait(false);

        return activity;
    }

    public async Task UpdateAsync(Activity activity)
    {
        ArgumentNullException.ThrowIfNull(activity);
        // Find the existing activity to determine its current week
        var existingActivity = await GetByIdAsync(activity.Id).ConfigureAwait(false);
        if (existingActivity == null)
            return;

        var oldWeekKey = WeekHelper.GetWeekKey(existingActivity.When);
        var newWeekKey = WeekHelper.GetWeekKey(activity.When);

        if (oldWeekKey == newWeekKey)
        {
            // Activity stays in the same week
            var weekActivities = await _storageService.GetActivitiesByWeekAsync(oldWeekKey).ConfigureAwait(false);
            // Remove any existing entries with this ID first to prevent duplicates
            weekActivities.RemoveAll(a => a.Id == activity.Id);
            // Add the updated activity
            weekActivities.Add(activity);
            await _storageService.SetActivitiesForWeekAsync(oldWeekKey, weekActivities).ConfigureAwait(false);
        }
        else
        {
            // Activity moved to a different week - remove from old week, add to new week
            var oldWeekActivities = await _storageService.GetActivitiesByWeekAsync(oldWeekKey).ConfigureAwait(false);
            oldWeekActivities.RemoveAll(a => a.Id == activity.Id);
            
            // If the old week is now empty, delete the week key; otherwise save the updated list
            if (oldWeekActivities.Count == 0)
            {
                await _storageService.RemoveActivitiesForWeekAsync(oldWeekKey).ConfigureAwait(false);
            }
            else
            {
                await _storageService.SetActivitiesForWeekAsync(oldWeekKey, oldWeekActivities).ConfigureAwait(false);
            }

            var newWeekActivities = await _storageService.GetActivitiesByWeekAsync(newWeekKey).ConfigureAwait(false);
            // Remove any existing entries with this ID first to prevent duplicates
            newWeekActivities.RemoveAll(a => a.Id == activity.Id);
            // Add the updated activity
            newWeekActivities.Add(activity);
            await _storageService.SetActivitiesForWeekAsync(newWeekKey, newWeekActivities).ConfigureAwait(false);
        }
    }

    public async Task DeleteAsync(int id)
    {
        // Find the activity to determine which week it's in
        var activity = await GetByIdAsync(id).ConfigureAwait(false);
        if (activity == null)
            return;

        var weekKey = WeekHelper.GetWeekKey(activity.When);
        var weekActivities = await _storageService.GetActivitiesByWeekAsync(weekKey).ConfigureAwait(false);
        weekActivities.RemoveAll(a => a.Id == id);
        await _storageService.SetActivitiesForWeekAsync(weekKey, weekActivities).ConfigureAwait(false);
    }

    public async Task<List<Activity>> GetByActivityTypeIdAsync(int activityTypeId)
    {
        var activities = await GetAllAsync().ConfigureAwait(false);
        return activities.Where(a => a.ActivityTypeId == activityTypeId).ToList();
    }

    public async Task<List<string>> GetAllAvailableWeekKeysAsync()
    {
        return await _storageService.GetAllAvailableWeekKeysAsync().ConfigureAwait(false);
    }

    public async Task RecalculateNextIdAsync()
    {
        var activities = await _storageService.GetItemAsync<List<Activity>>(StorageKey).ConfigureAwait(false) ?? new List<Activity>();
        if (activities.Count > 0)
        {
            _nextId = activities.Max(a => a.Id) + 1;
        }
        else
        {
            _nextId = 1;
        }
        // Persist the recalculated value
        await _storageService.SetItemAsync(NextIdKey, _nextId).ConfigureAwait(false);
        _nextIdInitialized = true;
    }
}
