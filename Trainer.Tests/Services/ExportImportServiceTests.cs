using Moq;
using Trainer.Models;
using Trainer.Services;
using System.Text.Json;

namespace Trainer.Tests.Services;

public class ExportImportServiceTests
{
    private readonly Mock<IStorageService> _storageServiceMock;
    private readonly ExportImportService _service;

    public ExportImportServiceTests()
    {
        _storageServiceMock = new Mock<IStorageService>();
        _service = new ExportImportService(_storageServiceMock.Object);
    }

    [Fact]
    public async Task ExportDataAsync_ReturnsJsonWithActivitiesAndTypes()
    {
        // Arrange
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test" }
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
    public async Task ImportDataAsync_ImportsActivitiesAndTypes()
    {
        // Arrange
        var exportData = new
        {
            activities = new List<Activity>
            {
                new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test" }
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
    }
}

