using Microsoft.JSInterop;
using System.Text.Json;

namespace Trainer.Services;

public class IndexedDbStorageService : IStorageService
{
    private readonly IJSRuntime _jsRuntime;
    private readonly JsonSerializerOptions _jsonOptions;
    private bool _isInitialized = false;
    private readonly SemaphoreSlim _initSemaphore = new(1, 1);
    private const string ActivitiesKey = "activities";

    public IndexedDbStorageService(IJSRuntime jsRuntime)
    {
        _jsRuntime = jsRuntime;
        _jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false
        };
    }

    private async Task EnsureInitializedAsync()
    {
        if (_isInitialized)
            return;

        await _initSemaphore.WaitAsync();
        try
        {
            if (_isInitialized)
                return;

            // Check if script is loaded - verify the object exists
            try
            {
                // Try to call a method to verify the script is loaded
                // We use a test key that likely doesn't exist - we just want to verify the function exists
                await _jsRuntime.InvokeAsync<string?>("indexedDbStorage.getItem", "__init_check__");
            }
            catch (JSException)
            {
                throw new InvalidOperationException("IndexedDB storage JavaScript is not loaded. Make sure indexeddb-storage.js is included in index.html.");
            }

            // Perform migration if needed
            await MigrateFromLocalStorageAsync();

            _isInitialized = true;
        }
        finally
        {
            _initSemaphore.Release();
        }
    }

    private async Task MigrateFromLocalStorageAsync()
    {
        try
        {
            // Migrate activities
            var existingActivitiesData = await _jsRuntime.InvokeAsync<string?>("localStorage.getItem", ActivitiesKey);
            if (!string.IsNullOrEmpty(existingActivitiesData))
            {
                // Parse activities from localStorage
                var activities = JsonSerializer.Deserialize<List<Models.Activity>>(existingActivitiesData, _jsonOptions);
                if (activities != null && activities.Any())
                {
                    // Group activities by week
                    var activitiesByWeek = activities
                        .GroupBy(a => WeekHelper.GetWeekKey(a.When))
                        .ToDictionary(g => g.Key, g => g.ToList());

                    // Store each week in IndexedDB
                    foreach (var weekGroup in activitiesByWeek)
                    {
                        var storageKey = WeekHelper.GetStorageKey(weekGroup.Key);
                        var json = JsonSerializer.Serialize(weekGroup.Value, _jsonOptions);
                        await _jsRuntime.InvokeVoidAsync("indexedDbStorage.setItem", storageKey, json);
                    }

                    // Remove from localStorage after successful migration
                    await _jsRuntime.InvokeVoidAsync("localStorage.removeItem", ActivitiesKey);
                }
            }

            // Migrate activityTypes
            var existingActivityTypesData = await _jsRuntime.InvokeAsync<string?>("localStorage.getItem", "activityTypes");
            if (!string.IsNullOrEmpty(existingActivityTypesData))
            {
                // Parse activityTypes from localStorage
                var activityTypes = JsonSerializer.Deserialize<List<Models.ActivityType>>(existingActivityTypesData, _jsonOptions);
                if (activityTypes != null && activityTypes.Any())
                {
                    // Store activityTypes in IndexedDB (not by week, just as a single item)
                    var json = JsonSerializer.Serialize(activityTypes, _jsonOptions);
                    await _jsRuntime.InvokeVoidAsync("indexedDbStorage.setItem", "activityTypes", json);

                    // Remove from localStorage after successful migration
                    await _jsRuntime.InvokeVoidAsync("localStorage.removeItem", "activityTypes");
                }
            }
        }
        catch (Exception ex)
        {
            // Log error but don't fail initialization
            Console.WriteLine($"Migration from localStorage failed: {ex.Message}");
        }
    }

    public virtual async Task<T?> GetItemAsync<T>(string key)
    {
        await EnsureInitializedAsync();

        // Special handling for activities key - return all activities from all weeks
        if (key == ActivitiesKey)
        {
            return await GetAllActivitiesAsync<T>();
        }

        // Normal handling for other keys
        try
        {
            var json = await _jsRuntime.InvokeAsync<string?>("indexedDbStorage.getItem", key);
            if (string.IsNullOrEmpty(json))
                return default(T);

            return JsonSerializer.Deserialize<T>(json, _jsonOptions);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Error getting item from IndexedDB for key '{key}': {ex.Message}");
            return default(T);
        }
    }

    private async Task<T?> GetAllActivitiesAsync<T>()
    {
        try
        {
            // Get all keys with activities- prefix
            var keys = await _jsRuntime.InvokeAsync<string[]>("indexedDbStorage.getAllKeysWithPrefix", "activities-");
            
            if (keys == null || keys.Length == 0)
                return JsonSerializer.Deserialize<T>("[]", _jsonOptions);

            // Get all week buckets
            // Serialize keys array to JSON to ensure it's passed correctly to JavaScript
            var keysJson = JsonSerializer.Serialize(keys, _jsonOptions);
            var itemsJson = await _jsRuntime.InvokeAsync<string>("indexedDbStorage.getItems", keysJson);
            
            if (string.IsNullOrEmpty(itemsJson))
                return JsonSerializer.Deserialize<T>("[]", _jsonOptions);

            // Deserialize the dictionary - keys are storage keys, values are List<Activity>
            var itemsDict = JsonSerializer.Deserialize<Dictionary<string, JsonElement>>(itemsJson, _jsonOptions);

            if (itemsDict == null || itemsDict.Count == 0)
                return JsonSerializer.Deserialize<T>("[]", _jsonOptions);

            // Deserialize each week's activities and flatten
            var allActivities = new List<Models.Activity>();
            foreach (var kvp in itemsDict)
            {
                var weekActivities = JsonSerializer.Deserialize<List<Models.Activity>>(kvp.Value.GetRawText(), _jsonOptions);
                if (weekActivities != null)
                {
                    allActivities.AddRange(weekActivities);
                }
            }

            // Serialize as the requested type
            var json = JsonSerializer.Serialize(allActivities, _jsonOptions);
            return JsonSerializer.Deserialize<T>(json, _jsonOptions);
        }
        catch (Exception ex)
        {
            // Log error for debugging
            Console.Error.WriteLine($"GetAllActivitiesAsync error: {ex.Message}");
            return JsonSerializer.Deserialize<T>("[]", _jsonOptions);
        }
    }

    public virtual async Task SetItemAsync<T>(string key, T value)
    {
        await EnsureInitializedAsync();

        // Special handling for activities - store by week
        if (key == ActivitiesKey && value is List<Models.Activity> activities)
        {
            await SetActivitiesAsync(activities);
            return;
        }

        // Normal handling for other keys
        var json = JsonSerializer.Serialize(value, _jsonOptions);
        await _jsRuntime.InvokeVoidAsync("indexedDbStorage.setItem", key, json);
    }

    private async Task SetActivitiesAsync(List<Models.Activity> activities)
    {
        // Group activities by week
        var activitiesByWeek = activities
            .GroupBy(a => WeekHelper.GetWeekKey(a.When))
            .ToDictionary(g => g.Key, g => g.ToList());

        // Get all existing week keys
        var existingKeys = await _jsRuntime.InvokeAsync<string[]>("indexedDbStorage.getAllKeysWithPrefix", "activities-");
        var existingKeysSet = new HashSet<string>(existingKeys ?? Array.Empty<string>());

        // Store or update each week bucket
        foreach (var weekGroup in activitiesByWeek)
        {
            var storageKey = WeekHelper.GetStorageKey(weekGroup.Key);
            var json = JsonSerializer.Serialize(weekGroup.Value, _jsonOptions);
            await _jsRuntime.InvokeVoidAsync("indexedDbStorage.setItem", storageKey, json);
            existingKeysSet.Remove(storageKey);
        }

        // Remove week buckets that no longer have any activities
        foreach (var emptyKey in existingKeysSet)
        {
            await _jsRuntime.InvokeVoidAsync("indexedDbStorage.removeItem", emptyKey);
        }
    }

    public virtual async Task RemoveItemAsync(string key)
    {
        await EnsureInitializedAsync();

        // Special handling for activities - remove all week buckets
        if (key == ActivitiesKey)
        {
            var keys = await _jsRuntime.InvokeAsync<string[]>("indexedDbStorage.getAllKeysWithPrefix", "activities-");
            if (keys != null)
            {
                foreach (var weekKey in keys)
                {
                    await _jsRuntime.InvokeVoidAsync("indexedDbStorage.removeItem", weekKey);
                }
            }
            return;
        }

        // Normal handling for other keys
        await _jsRuntime.InvokeVoidAsync("indexedDbStorage.removeItem", key);
    }

    public virtual async Task ClearAsync()
    {
        await EnsureInitializedAsync();
        await _jsRuntime.InvokeVoidAsync("indexedDbStorage.clear");
    }

    // Helper methods for ActivityService to work with weekly storage directly
    public virtual async Task<List<Models.Activity>> GetActivitiesByWeekAsync(string weekKey)
    {
        await EnsureInitializedAsync();
        var storageKey = WeekHelper.GetStorageKey(weekKey);
        var json = await _jsRuntime.InvokeAsync<string?>("indexedDbStorage.getItem", storageKey);
        
        if (string.IsNullOrEmpty(json))
            return new List<Models.Activity>();

        return JsonSerializer.Deserialize<List<Models.Activity>>(json, _jsonOptions) ?? new List<Models.Activity>();
    }

    public virtual async Task<List<Models.Activity>> GetActivitiesByWeekRangeAsync(IEnumerable<string> weekKeys)
    {
        await EnsureInitializedAsync();
        var storageKeys = weekKeys.Select(WeekHelper.GetStorageKey).ToArray();
        // Serialize keys array to JSON to ensure it's passed correctly to JavaScript
        var keysJson = JsonSerializer.Serialize(storageKeys, _jsonOptions);
        var itemsJson = await _jsRuntime.InvokeAsync<string>("indexedDbStorage.getItems", keysJson);
        
        if (string.IsNullOrEmpty(itemsJson))
            return new List<Models.Activity>();

        // Deserialize to Dictionary<string, JsonElement> first (values are nested JSON)
        var itemsDict = JsonSerializer.Deserialize<Dictionary<string, JsonElement>>(itemsJson, _jsonOptions);

        if (itemsDict == null || itemsDict.Count == 0)
            return new List<Models.Activity>();

        // Deserialize each week's activities and flatten
        var allActivities = new List<Models.Activity>();
        foreach (var kvp in itemsDict)
        {
            var weekActivities = JsonSerializer.Deserialize<List<Models.Activity>>(kvp.Value.GetRawText(), _jsonOptions);
            if (weekActivities != null)
            {
                allActivities.AddRange(weekActivities);
            }
        }

        return allActivities;
    }

    public virtual async Task SetActivitiesForWeekAsync(string weekKey, List<Models.Activity> activities)
    {
        await EnsureInitializedAsync();
        var storageKey = WeekHelper.GetStorageKey(weekKey);
        var json = JsonSerializer.Serialize(activities, _jsonOptions);
        await _jsRuntime.InvokeVoidAsync("indexedDbStorage.setItem", storageKey, json);
    }

    public virtual async Task RemoveActivitiesForWeekAsync(string weekKey)
    {
        await EnsureInitializedAsync();
        var storageKey = WeekHelper.GetStorageKey(weekKey);
        await _jsRuntime.InvokeVoidAsync("indexedDbStorage.removeItem", storageKey);
    }

    /// <summary>
    /// Gets all available week keys that have activities stored in IndexedDB
    /// </summary>
    public virtual async Task<List<string>> GetAllAvailableWeekKeysAsync()
    {
        await EnsureInitializedAsync();
        try
        {
            // Get all keys with activities- prefix
            var storageKeys = await _jsRuntime.InvokeAsync<string[]>("indexedDbStorage.getAllKeysWithPrefix", "activities-");
            
            if (storageKeys == null || storageKeys.Length == 0)
                return new List<string>();

            // Extract week keys from storage keys (remove "activities-" prefix)
            var weekKeys = storageKeys
                .Select(WeekHelper.ExtractWeekKey)
                .Where(wk => !string.IsNullOrEmpty(wk))
                .ToList();

            return weekKeys;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"GetAllAvailableWeekKeysAsync error: {ex.Message}");
            return new List<string>();
        }
    }
}

