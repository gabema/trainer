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
        var activities = await _storageService.GetItemAsync<List<Activity>>("activities") ?? new List<Activity>();
        var activityTypes = await _storageService.GetItemAsync<List<ActivityType>>("activityTypes") ?? new List<ActivityType>();

        var exportData = new
        {
            Activities = activities,
            ActivityTypes = activityTypes,
            ExportDate = DateTime.UtcNow
        };

        return JsonSerializer.Serialize(exportData, _jsonOptions);
    }

    public async Task ImportDataAsync(string jsonData)
    {
        try
        {
            var importData = JsonSerializer.Deserialize<ImportData>(jsonData, _jsonOptions);
            
            if (importData?.Activities != null)
            {
                await _storageService.SetItemAsync("activities", importData.Activities);
            }

            if (importData?.ActivityTypes != null)
            {
                await _storageService.SetItemAsync("activityTypes", importData.ActivityTypes);
            }
        }
        catch
        {
            throw new InvalidOperationException("Invalid import data format");
        }
    }

    private class ImportData
    {
        public List<Activity>? Activities { get; set; }
        public List<ActivityType>? ActivityTypes { get; set; }
    }
}

