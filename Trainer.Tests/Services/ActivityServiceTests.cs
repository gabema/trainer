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

    [Fact]
    public async Task EnsureNextIdInitializedAsync_LoadsFromLocalStorage_WhenExists()
    {
        // Arrange
        const int storedNextId = 42;
        _storageServiceMock
            .Setup(x => x.GetItemAsync<int?>("activityNextId"))
            .ReturnsAsync(storedNextId);

        var activities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        var newActivity = new Activity { ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "New" };
        var weekKey = WeekHelper.GetWeekKey(newActivity.When);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(new List<Activity>());

        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        // Act
        var result = await _service.AddAsync(newActivity);

        // Assert
        Assert.Equal(storedNextId, result.Id);
        _storageServiceMock.Verify(x => x.GetItemAsync<int?>("activityNextId"), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", storedNextId + 1), Times.Once);
    }

    [Fact]
    public async Task EnsureNextIdInitializedAsync_CalculatesFromActivities_WhenLocalStorageEmpty()
    {
        // Arrange
        _storageServiceMock
            .Setup(x => x.GetItemAsync<int?>("activityNextId"))
            .ReturnsAsync((int?)null);

        var activities = new List<Activity>
        {
            new() { Id = 5, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test 1" },
            new() { Id = 10, ActivityTypeId = 1, When = DateTime.Now, Amount = 200, Notes = "Test 2" }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        var newActivity = new Activity { ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "New" };
        var weekKey = WeekHelper.GetWeekKey(newActivity.When);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(new List<Activity>());

        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        // Act
        var result = await _service.AddAsync(newActivity);

        // Assert
        Assert.Equal(11, result.Id); // Max ID (10) + 1
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", 11), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", 12), Times.Once); // Once for calculation, once for increment
    }

    [Fact]
    public async Task EnsureNextIdInitializedAsync_DefaultsToOne_WhenNoActivitiesAndNoLocalStorage()
    {
        // Arrange
        _storageServiceMock
            .Setup(x => x.GetItemAsync<int?>("activityNextId"))
            .ReturnsAsync((int?)null);

        var activities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        var newActivity = new Activity { ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "New" };
        var weekKey = WeekHelper.GetWeekKey(newActivity.When);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(new List<Activity>());

        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        // Act
        var result = await _service.AddAsync(newActivity);

        // Assert
        Assert.Equal(1, result.Id);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", 1), Times.Once);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", 2), Times.Once); // Once for default, once for increment
    }

    [Fact]
    public async Task AddAsync_PersistsNextId_AfterIncrementing()
    {
        // Arrange
        const int initialNextId = 5;
        _storageServiceMock
            .Setup(x => x.GetItemAsync<int?>("activityNextId"))
            .ReturnsAsync(initialNextId);

        var activities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        var newActivity = new Activity { ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "New" };
        var weekKey = WeekHelper.GetWeekKey(newActivity.When);

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(weekKey))
            .ReturnsAsync(new List<Activity>());

        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(weekKey, It.IsAny<List<Activity>>()))
            .Returns(Task.CompletedTask);

        // Act
        var result = await _service.AddAsync(newActivity);

        // Assert
        Assert.Equal(initialNextId, result.Id);
        // Verify nextId was persisted after incrementing (should be initialNextId + 1)
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", initialNextId + 1), Times.Once);
    }

    [Fact]
    public async Task RecalculateNextIdAsync_UpdatesNextId_FromAllActivities()
    {
        // Arrange
        var activities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = DateTime.Now, Amount = 100, Notes = "Test 1" },
            new() { Id = 15, ActivityTypeId = 1, When = DateTime.Now, Amount = 200, Notes = "Test 2" },
            new() { Id = 7, ActivityTypeId = 1, When = DateTime.Now, Amount = 300, Notes = "Test 3" }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        // Act
        await _service.RecalculateNextIdAsync();

        // Assert
        // Should calculate max ID (15) + 1 = 16
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", 16), Times.Once);
    }

    [Fact]
    public async Task RecalculateNextIdAsync_SetsToOne_WhenNoActivities()
    {
        // Arrange
        var activities = new List<Activity>();
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(activities);

        // Act
        await _service.RecalculateNextIdAsync();

        // Assert
        _storageServiceMock.Verify(x => x.SetItemAsync("activityNextId", 1), Times.Once);
    }

    [Fact]
    public async Task AddAsync_AfterImport_RecalculatesNextId()
    {
        // Arrange
        var testDate = DateTime.Now;
        var weekKey = WeekHelper.GetWeekKey(testDate);
        
        // Step 1: Initialize with activities 1-5 to set _nextId = 6
        var initialActivities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = testDate, Amount = 100, Notes = "Initial 1" },
            new() { Id = 2, ActivityTypeId = 1, When = testDate, Amount = 200, Notes = "Initial 2" },
            new() { Id = 3, ActivityTypeId = 1, When = testDate, Amount = 300, Notes = "Initial 3" },
        };

        // Step 2: Import activities with IDs 20-25 (simulating import)
        var importedActivities = new List<Activity>
        {
            new() { Id = 20, ActivityTypeId = 1, When = testDate, Amount = 2000, Notes = "Imported 20" },
            new() { Id = 21, ActivityTypeId = 1, When = testDate, Amount = 2100, Notes = "Imported 21" },
            new() { Id = 22, ActivityTypeId = 1, When = testDate, Amount = 2200, Notes = "Imported 22" },
        };

        // Track the current state of activities
        var currentActivities = new List<Activity>(initialActivities);
        var savedActivitiesByWeek = new Dictionary<string, List<Activity>>();

        // Setup mock to return initial activities first, then all activities after "import"
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<Activity>>("activities"))
            .ReturnsAsync(() => new List<Activity>(currentActivities));

        _storageServiceMock
            .Setup(x => x.GetActivitiesByWeekAsync(It.IsAny<string>()))
            .ReturnsAsync((string wk) => 
            {
                if (savedActivitiesByWeek.TryGetValue(wk, out var weekActivities))
                {
                    return new List<Activity>(weekActivities);
                }
                // Return activities for this week from currentActivities
                return currentActivities.Where(a => WeekHelper.GetWeekKey(a.When) == wk).ToList();
            });

        _storageServiceMock
            .Setup(x => x.SetActivitiesForWeekAsync(It.IsAny<string>(), It.IsAny<List<Activity>>()))
            .Callback<string, List<Activity>>((wk, list) => 
            { 
                savedActivitiesByWeek[wk] = new List<Activity>(list);
                // Update currentActivities to reflect the saved state
                currentActivities = savedActivitiesByWeek.Values.SelectMany(x => x).Concat(
                    currentActivities.Where(a => !savedActivitiesByWeek.Values.SelectMany(x => x).Any(saved => saved.Id == a.Id))
                ).ToList();
            })
            .Returns(Task.CompletedTask);

        // Step 3: Initialize _nextId by calling GetAllAsync (this sets _nextId = 4 based on initial activities)
        // Mock localStorage to return null initially (no stored nextId)
        _storageServiceMock
            .Setup(x => x.GetItemAsync<int?>("activityNextId"))
            .ReturnsAsync((int?)null);

        await _service.GetAllAsync();

        // Step 4: Simulate import by updating storage
        currentActivities = initialActivities.Concat(importedActivities).ToList();
        savedActivitiesByWeek[weekKey] = new List<Activity>(currentActivities);

        // Step 5: Recalculate nextId after import (this is what ExportImportService would do)
        await _service.RecalculateNextIdAsync();

        // Step 6: Add new activities - they should get IDs starting from 23 (max imported ID 22 + 1)
        var newActivities = new List<Activity>();
        for (int i = 0; i < 3; i++)
        {
            var newActivity = new Activity 
            { 
                ActivityTypeId = 1, 
                When = testDate.AddDays(i), 
                Amount = 100 + i, 
                Notes = $"New {i}" 
            };
            var added = await _service.AddAsync(newActivity);
            newActivities.Add(added);
        }

        // Assert: Verify no ID collisions - all IDs should be unique
        var allActivities = await _service.GetAllAsync();
        var activityIds = allActivities.Select(a => a.Id).ToList();
        var uniqueIds = activityIds.Distinct().ToList();
    
        // Verify no duplicates
        Assert.Equal(activityIds.Count, uniqueIds.Count);
    
        // Verify that new activities got IDs starting from 23
        Assert.Equal(23, newActivities[0].Id);
        Assert.Equal(24, newActivities[1].Id);
        Assert.Equal(25, newActivities[2].Id);
    
        // Verify that imported activities still have their original IDs
        var importedActivity = allActivities.FirstOrDefault(a => a.Id == 20 && a.Notes == "Imported 20");
        Assert.NotNull(importedActivity);
    }
}

