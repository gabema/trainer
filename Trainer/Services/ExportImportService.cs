using System.Text.Json;
using Trainer.Models;

namespace Trainer.Services;

public class ExportImportService : IExportImportService
{
    private readonly IStorageService _storageService;
    private readonly JsonSerializerOptions _jsonOptions;

    public ExportImportService(IStorageService storageService)
    {
        _storageService = storageService;
        _jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = true
        };
    }

    public async Task<string> ExportDataAsync()
    {
        var activityTypes = await _storageService.GetItemAsync<List<ActivityType>>("activityTypes") ?? new List<ActivityType>();

        // Export activities in weekly format
        Dictionary<string, List<Activity>> activitiesByWeek;
        
        if (_storageService is IndexedDbStorageService indexedDbService)
        {
            // Get all week keys
            var allActivities = await _storageService.GetItemAsync<List<Activity>>("activities") ?? new List<Activity>();
            
            // Group by week for export format
            activitiesByWeek = allActivities
                .GroupBy(a => WeekHelper.GetWeekKey(a.When))
                .ToDictionary(g => g.Key, g => g.ToList());
        }
        else
        {
            // Fallback for other storage services - group existing activities
            var activities = await _storageService.GetItemAsync<List<Activity>>("activities") ?? new List<Activity>();
            activitiesByWeek = activities
                .GroupBy(a => WeekHelper.GetWeekKey(a.When))
                .ToDictionary(g => g.Key, g => g.ToList());
        }

        var exportData = new
        {
            Activities = activitiesByWeek,
            ActivityTypes = activityTypes,
            ExportDate = DateTime.UtcNow
        };

        return JsonSerializer.Serialize(exportData, _jsonOptions);
    }

    public async Task ImportDataAsync(string jsonData)
    {
        try
        {
            using var jsonDoc = JsonDocument.Parse(jsonData);
            var root = jsonDoc.RootElement;

            // Handle activities - support both old format (array) and new format (weekly object)
            if (root.TryGetProperty("activities", out var activitiesElement))
            {
                if (activitiesElement.ValueKind == JsonValueKind.Array)
                {
                    // Old format: activities is an array
                    var activities = JsonSerializer.Deserialize<List<Activity>>(activitiesElement, _jsonOptions);
                    if (activities != null)
                    {
                        await _storageService.SetItemAsync("activities", activities);
                    }
                }
                else if (activitiesElement.ValueKind == JsonValueKind.Object)
                {
                    // New format: activities is an object with week keys
                    var activitiesByWeek = JsonSerializer.Deserialize<Dictionary<string, List<Activity>>>(activitiesElement, _jsonOptions);
                    if (activitiesByWeek != null && _storageService is IndexedDbStorageService indexedDbService)
                    {
                        // Store each week's activities
                        foreach (var weekGroup in activitiesByWeek)
                        {
                            await indexedDbService.SetActivitiesForWeekAsync(weekGroup.Key, weekGroup.Value);
                        }
                    }
                    else if (activitiesByWeek != null)
                    {
                        // Flatten and store for non-IndexedDB storage
                        var allActivities = activitiesByWeek.Values.SelectMany(x => x).ToList();
                        await _storageService.SetItemAsync("activities", allActivities);
                    }
                }
            }
            else if (root.TryGetProperty("Activities", out var activitiesElementPascal))
            {
                // Try with PascalCase property name (backward compatibility)
                if (activitiesElementPascal.ValueKind == JsonValueKind.Array)
                {
                    var activities = JsonSerializer.Deserialize<List<Activity>>(activitiesElementPascal, _jsonOptions);
                    if (activities != null)
                    {
                        await _storageService.SetItemAsync("activities", activities);
                    }
                }
                else if (activitiesElementPascal.ValueKind == JsonValueKind.Object)
                {
                    var activitiesByWeek = JsonSerializer.Deserialize<Dictionary<string, List<Activity>>>(activitiesElementPascal, _jsonOptions);
                    if (activitiesByWeek != null && _storageService is IndexedDbStorageService indexedDbService)
                    {
                        foreach (var weekGroup in activitiesByWeek)
                        {
                            await indexedDbService.SetActivitiesForWeekAsync(weekGroup.Key, weekGroup.Value);
                        }
                    }
                    else if (activitiesByWeek != null)
                    {
                        var allActivities = activitiesByWeek.Values.SelectMany(x => x).ToList();
                        await _storageService.SetItemAsync("activities", allActivities);
                    }
                }
            }

            // Handle activity types
            if (root.TryGetProperty("activityTypes", out var activityTypesElement))
            {
                var activityTypes = JsonSerializer.Deserialize<List<ActivityType>>(activityTypesElement, _jsonOptions);
                if (activityTypes != null)
                {
                    await _storageService.SetItemAsync("activityTypes", activityTypes);
                }
            }
            else if (root.TryGetProperty("ActivityTypes", out var activityTypesElementPascal))
            {
                var activityTypes = JsonSerializer.Deserialize<List<ActivityType>>(activityTypesElementPascal, _jsonOptions);
                if (activityTypes != null)
                {
                    await _storageService.SetItemAsync("activityTypes", activityTypes);
                }
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Invalid import data format: {ex.Message}", ex);
        }
    }
}

