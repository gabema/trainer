using Moq;
using Trainer.Models;
using Trainer.Services;

namespace Trainer.Tests.Services;

public class ActivityServiceTests
{
    private readonly Mock<IndexedDbStorageService> _storageServiceMock;
    private readonly ActivityService _service;

    public ActivityServiceTests()
    {
        _storageServiceMock = new Mock<IndexedDbStorageService>(Mock.Of<Microsoft.JSInterop.IJSRuntime>());
        _service = new ActivityService(_storageServiceMock.Object);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsEmptyList_WhenNoActivitiesExist()
    {
        // Arrange
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync((List<Activity>?)null);

        // Act
        var result = await _service.GetAllAsync();

        // Assert
        Assert.NotNull(result);
        Assert.Empty(result);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsAllActivities()
    {
        // Arrange
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test 1" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 200, Notes = "Test 2" }
        };

        // Set up mock directly on IndexedDbStorageService (which implements IStorageService)
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        // Act
        var result = await _service.GetAllAsync();

        // Assert
        Assert.NotNull(result);
        Assert.Equal(2, result.Count);
    }

    [Fact]
    public async Task GetByIdAsync_ReturnsActivity_WhenExists()
    {
        // Arrange
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test 1" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 200, Notes = "Test 2" }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        // Act
        var result = await _service.GetByIdAsync(1);

        // Assert
        Assert.NotNull(result);
        Assert.Equal(1, result.Id);
        Assert.Equal("Test 1", result.Notes);
    }

    [Fact]
    public async Task GetByIdAsync_ReturnsNull_WhenNotExists()
    {
        // Arrange
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test 1" }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        // Act
        var result = await _service.GetByIdAsync(999);

        // Assert
        Assert.Null(result);
    }

    [Fact]
    public async Task AddAsync_AddsActivityWithNewId()
    {
        // Arrange
        var activities = new List<Activity>();
        var newActivity = new Activity { ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "New" };
        var weekKey = WeekHelper.GetWeekKey(newActivity.When);

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(new List<Activity>());

        var savedActivities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Callback<string, List<Activity>>((wk, list) => { savedActivities.Clear(); savedActivities.AddRange(list); })
            .Returns(Task.CompletedTask);

        // Act
        var result = await _service.AddAsync(newActivity);

        // Assert
        Assert.True(result.Id > 0);
        Assert.Single(savedActivities);
        _storageServiceMock.Verify(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()), Times.Once);
    }

    [Fact]
    public async Task UpdateAsync_UpdatesExistingActivity()
    {
        // Arrange
        var testDate = DateTime.Now;
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Original" }
        };
        var weekKey = WeekHelper.GetWeekKey(testDate);
        var updatedActivity = new Activity { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 200, Notes = "Updated" };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(activities);

        var savedActivities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Callback<string, List<Activity>>((wk, list) => { savedActivities.Clear(); savedActivities.AddRange(list); })
            .Returns(Task.CompletedTask);

        // Act
        await _service.UpdateAsync(updatedActivity);

        // Assert
        Assert.Single(savedActivities);
        Assert.Equal("Updated", savedActivities[0].Notes);
        Assert.Equal(200, savedActivities[0].Amount);
        _storageServiceMock.Verify(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()), Times.Once);
    }

    [Fact]
    public async Task DeleteAsync_RemovesActivity()
    {
        // Arrange
        var testDate = DateTime.Now;
        var weekKey = WeekHelper.GetWeekKey(testDate);
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Test 1" },
            new() { Id = 2, ActivityTypeId = 2, When = testDate, Amount = 200, Notes = "Test 2" }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(activities);

        var savedActivities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Callback<string, List<Activity>>((wk, list) => { savedActivities.Clear(); savedActivities.AddRange(list); })
            .Returns(Task.CompletedTask);

        // Act
        await _service.DeleteAsync(1);

        // Assert
        Assert.Single(savedActivities);
        Assert.Equal(2, savedActivities.First().Id);
        _storageServiceMock.Verify(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()), Times.Once);
    }

    [Fact]
    public async Task GetByActivityTypeIdAsync_ReturnsFilteredActivities()
    {
        // Arrange
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Type 1" },
            new() { Id = 2, ActivityTypeId = 2, When = DateTime.Now, Amount = 200, Notes = "Type 2" },
            new() { Id = 3, ActivityTypeId = 1, When = DateTime.Now, Amount = 300, Notes = "Type 1 again" }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        // Act
        var result = await _service.GetByActivityTypeIdAsync(1);

        // Assert
        Assert.NotNull(result);
        Assert.Equal(2, result.Count);
        Assert.All(result, a => Assert.Equal(1, a.ActivityTypeId));
    }
}

