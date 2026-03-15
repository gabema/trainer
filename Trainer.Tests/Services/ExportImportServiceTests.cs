namespace Trainer.Tests.Services;

using Moq;
using Trainer.Models;
using Trainer.Services;
using System.Text.Json;
using Trainer.Serialization;

public class ExportImportServiceTests
{
    private readonly Mock<IStorageService> _storageServiceMock;
    private readonly Mock<IActivityService> _activityServiceMock;
    private readonly ExportImportService _service;

    public ExportImportServiceTests()
    {
        _storageServiceMock = new Mock<IStorageService>();
        _activityServiceMock = new Mock<IActivityService>();
        _service = new ExportImportService(_storageServiceMock.Object, _activityServiceMock.Object);
    }

    [Fact]
    public async Task ExportDataAsync_ReturnsJsonWithActivitiesAndTypes()
    {
        // Arrange
        var testDate = DateTime.Now;
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Test", DurationSeconds = 90 }
        };

        var activityTypes = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(activityTypes);

        // Act
        var result = await _service.ExportDataAsync();

        // Assert
        Assert.NotNull(result);
        Assert.Contains("activities", result);
        Assert.Contains("activityTypes", result);
        Assert.Contains("exportDate", result);
        Assert.Contains("durationSeconds", result);
        
        // Verify weekly format
        var jsonDoc = JsonDocument.Parse(result);
        var activitiesElement = jsonDoc.RootElement.GetProperty("activities");
        Assert.Equal(JsonValueKind.Object, activitiesElement.ValueKind);
    }

    [Fact]
    public async Task ExportDataAsync_HandlesEmptyData()
    {
        // Arrange
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync((List<Activity>?)null);

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync((List<ActivityType>?)null);

        // Act
        var result = await _service.ExportDataAsync();

        // Assert
        Assert.NotNull(result);
        var jsonDoc = JsonDocument.Parse(result);
        Assert.True(jsonDoc.RootElement.TryGetProperty("activities", out _));
        Assert.True(jsonDoc.RootElement.TryGetProperty("activityTypes", out _));
    }

    [Fact]
    public async Task ImportDataAsync_ImportsActivitiesAndTypes_OldFormat()
    {
        // Arrange - Test old format (array)
        var testDate = DateTime.Now;
        var exportData = new
        {
            activities = new List<Activity>
            {
                new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Test", DurationSeconds = 150 }
            },
            activityTypes = new List<ActivityType>
            {
                new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive }
            }
        };

        var json = JsonSerializer.Serialize(exportData, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        // Act
        await _service.ImportDataAsync(json);

        // Assert
        _storageServiceMock.Verify(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Once);
        _activityServiceMock.Verify(x => x.RecalculateNextIdAsync(), Times.Once);
    }

    [Fact]
    public async Task ImportDataAsync_ImportsActivitiesAndTypes_NewFormat()
    {
        // Arrange - Test new format (weekly object)
        var testDate = DateTime.Now;
        var weekKey = WeekHelper.GetWeekKey(testDate);
        var exportData = new
        {
            activities = new Dictionary<string, List<Activity>>
            {
                { weekKey, new List<Activity> { new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Test", DurationSeconds = 300 } } }
            },
            activityTypes = new List<ActivityType>
            {
                new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive }
            }
        };

        var json = JsonSerializer.Serialize(exportData, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

        // Since ExportImportService checks if _storageService is IndexedDbStorageService,
        // and our mock is IStorageService, it will fall back to flattening and storing as a list
        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        // Act
        await _service.ImportDataAsync(json);

        // Assert - Since the mock is IStorageService (not IndexedDbStorageService), it uses SetItemAsync
        _storageServiceMock.Verify(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Once);
        _activityServiceMock.Verify(x => x.RecalculateNextIdAsync(), Times.Once);
    }

    [Fact]
    public async Task ImportDataAsync_ThrowsException_WhenInvalidJson()
    {
        // Arrange
        var invalidJson = "{ invalid json }";

        // Act & Assert
        await Assert.ThrowsAsync<InvalidOperationException>(() => _service.ImportDataAsync(invalidJson));
    }

    [Fact]
    public async Task ImportDataAsync_HandlesPartialData()
    {
        // Arrange
        var exportData = new
        {
            activities = new List<Activity>
            {
                new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test" }
            }
        };

        var json = JsonSerializer.Serialize(exportData, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        // Act
        await _service.ImportDataAsync(json);

        // Assert
        _storageServiceMock.Verify(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Never);
        _activityServiceMock.Verify(x => x.RecalculateNextIdAsync(), Times.Once);
    }

    [Fact]
    public async Task ImportDataAsync_RecalculatesNextId_AfterImportingActivities()
    {
        // Arrange
        var testDate = DateTime.Now;
        var exportData = new
        {
            activities = new List<Activity>
            {
                new() { Id = 10, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Imported 1" },
                new() { Id = 20, ActivityTypeId = 1, When = testDate, Amount = 200, Notes = "Imported 2" },
                new() { Id = 30, ActivityTypeId = 1, When = testDate, Amount = 300, Notes = "Imported 3" }
            },
            activityTypes = new List<ActivityType>
            {
                new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive }
            }
        };

        var json = JsonSerializer.Serialize(exportData, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        _activityServiceMock
            .Setup(x => x.RecalculateNextIdAsync())
            .Returns(Task.CompletedTask);

        // Act
        await _service.ImportDataAsync(json);

        // Assert
        _activityServiceMock.Verify(x => x.RecalculateNextIdAsync(), Times.Once, "RecalculateNextIdAsync should be called after importing activities");
    }

    [Fact]
    public async Task ImportDataAsync_DoesNotRecalculateNextId_WhenNoActivitiesImported()
    {
        // Arrange
        var exportData = new
        {
            activityTypes = new List<ActivityType>
            {
                new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive }
            }
        };

        var json = JsonSerializer.Serialize(exportData, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        // Act
        await _service.ImportDataAsync(json);

        // Assert
        _activityServiceMock.Verify(x => x.RecalculateNextIdAsync(), Times.Never, "RecalculateNextIdAsync should not be called when no activities are imported");
    }

    [Fact]
    public async Task ExportDataAsync_MinifiesOutput_OmitsNulls_AndRoundTrips()
    {
        // Arrange - data with optional null fields
        var testDate = DateTime.Now;
        var weekKey = WeekHelper.GetWeekKey(testDate);
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Test", DurationSeconds = null }
        };
        var activityTypes = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive, DailyAmount = null, WeeklyAmount = null, Unit = null }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(activityTypes);
        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);
        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        // Act - export
        var exported = await _service.ExportDataAsync();

        // Assert - minified: no newlines between properties
        Assert.NotNull(exported);
        Assert.DoesNotContain("\n", exported);
        // Assert - nulls omitted (no "key":null in output)
        Assert.DoesNotContain("\"durationSeconds\":null", exported);
        Assert.DoesNotContain("\"unit\":null", exported);
        Assert.DoesNotContain("\"dailyAmount\":null", exported);
        Assert.DoesNotContain("\"weeklyAmount\":null", exported);

        // Act - re-import the exported JSON
        await _service.ImportDataAsync(exported);

        // Assert - round-trip: import succeeded (storage was called with data)
        _storageServiceMock.Verify(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Once);
    }

    /// <summary>
    /// .NET DateTimeOffset.Parse does not support ISO 8601 hour-only offset (±hh); our converter
    /// normalizes to ±hh:00 in Read() so export/import round-trips for zero-minute timezones.
    /// </summary>
    [Fact]
    public async Task ImportDataAsync_ParsesHourOnlyTimezoneOffset()
    {
        // JSON with hour-only offset (e.g. -05 instead of -05:00) as produced by our exporter in most timezones
        var weekKey = "2026.11";
        var json = $$"""
            {"activities":{"{{weekKey}}":[{"id":1,"activityTypeId":1,"when":"2026-03-15T12:30:45-05","amount":10,"notes":"x"}],"2026.12":[]},"activityTypes":[{"id":1,"name":"Water","netBenefit":0}]}
            """;

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);
        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        await _service.ImportDataAsync(json);

        _storageServiceMock.Verify(x => x.SetItemAsync("activities", It.Is<List<Activity>>(list =>
            list.Count > 0 && list[0].When.Year == 2026 && list[0].When.Month == 3 && list[0].When.Day == 15)), Times.Once);
    }

    /// <summary>
    /// Activity.Notes is nullable; null or missing notes in JSON deserialize as null.
    /// </summary>
    [Fact]
    public async Task ImportDataAsync_DeserializesNullOrMissingNotesAsNull()
    {
        var weekKey = "2026.11";
        var fullJson = $$"""
            {"activities":{"{{weekKey}}":[{"id":1,"activityTypeId":1,"when":"2026-03-15T12:30:45Z","amount":5,"notes":null}],"2026.12":[]},"activityTypes":[{"id":1,"name":"Water","netBenefit":0}]}
            """;

        List<Activity>? captured = null;
        _storageServiceMock
            .Setup(x => x.SetItemAsync("activities", It.IsAny<List<Activity>>()))
            .Callback<string, List<Activity>>((_, list) => captured = list)
            .Returns(Task.CompletedTask);
        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Returns(Task.CompletedTask);

        await _service.ImportDataAsync(fullJson);

        Assert.NotNull(captured);
        Assert.Single(captured);
        Assert.Null(captured![0].Notes);
    }
}
