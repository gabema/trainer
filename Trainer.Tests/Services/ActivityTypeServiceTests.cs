using Moq;
using Trainer.Models;
using Trainer.Services;

namespace Trainer.Tests.Services;

public class ActivityTypeServiceTests
{
    private readonly Mock<IStorageService> _storageServiceMock;
    private readonly ActivityTypeService _service;

    public ActivityTypeServiceTests()
    {
        _storageServiceMock = new Mock<IStorageService>();
        _service = new ActivityTypeService(_storageServiceMock.Object);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsEmptyList_WhenNoTypesExist()
    {
        // Arrange
        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync((List<ActivityType>?)null);

        // Act
        var result = await _service.GetAllAsync();

        // Assert
        Assert.NotNull(result);
        Assert.Empty(result);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsAllActivityTypes()
    {
        // Arrange
        var types = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive },
            new() { Id = 2, Name = "Snack", NetBenefit = NetBenefit.Negative }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        // Act
        var result = await _service.GetAllAsync();

        // Assert
        Assert.NotNull(result);
        Assert.Equal(2, result.Count);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsActivityTypesSortedByName()
    {
        // Arrange
        var types = new List<ActivityType>
        {
            new() { Id = 1, Name = "Zebra", NetBenefit = NetBenefit.Positive },
            new() { Id = 2, Name = "Apple", NetBenefit = NetBenefit.Positive },
            new() { Id = 3, Name = "Banana", NetBenefit = NetBenefit.Negative },
            new() { Id = 4, Name = "Water", NetBenefit = NetBenefit.Positive }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        // Act
        var result = await _service.GetAllAsync();

        // Assert
        Assert.NotNull(result);
        Assert.Equal(4, result.Count);
        Assert.Equal("Apple", result[0].Name);
        Assert.Equal("Banana", result[1].Name);
        Assert.Equal("Water", result[2].Name);
        Assert.Equal("Zebra", result[3].Name);
    }

    [Fact]
    public async Task GetByIdAsync_ReturnsActivityType_WhenExists()
    {
        // Arrange
        var types = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive },
            new() { Id = 2, Name = "Snack", NetBenefit = NetBenefit.Negative }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        // Act
        var result = await _service.GetByIdAsync(1);

        // Assert
        Assert.NotNull(result);
        Assert.Equal(1, result.Id);
        Assert.Equal("Water", result.Name);
    }

    [Fact]
    public async Task GetByIdAsync_ReturnsNull_WhenNotExists()
    {
        // Arrange
        var types = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        // Act
        var result = await _service.GetByIdAsync(999);

        // Assert
        Assert.Null(result);
    }

    [Fact]
    public async Task AddAsync_AddsActivityTypeWithNewId()
    {
        // Arrange
        var types = new List<ActivityType>();
        var newType = new ActivityType { Name = "Exercise", NetBenefit = NetBenefit.Positive };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Callback<string, List<ActivityType>>((key, list) => types = list)
            .Returns(Task.CompletedTask);

        // Act
        var result = await _service.AddAsync(newType);

        // Assert
        Assert.True(result.Id > 0);
        Assert.Single(types);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Once);
    }

    [Fact]
    public async Task UpdateAsync_UpdatesExistingActivityType()
    {
        // Arrange
        var types = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive, DailyAmount = 8 }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Callback<string, List<ActivityType>>((key, list) => types = list)
            .Returns(Task.CompletedTask);

        var updatedType = new ActivityType { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive, DailyAmount = 10 };

        // Act
        await _service.UpdateAsync(updatedType);

        // Assert
        Assert.Equal(10, types[0].DailyAmount);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Once);
    }

    [Fact]
    public async Task DeleteAsync_RemovesActivityType()
    {
        // Arrange
        var types = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive },
            new() { Id = 2, Name = "Snack", NetBenefit = NetBenefit.Negative }
        };

        _storageServiceMock
            .Setup(x => x.GetItemAsync<List<ActivityType>>("activityTypes"))
            .ReturnsAsync(types);

        _storageServiceMock
            .Setup(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()))
            .Callback<string, List<ActivityType>>((key, list) => types = list)
            .Returns(Task.CompletedTask);

        // Act
        await _service.DeleteAsync(1);

        // Assert
        Assert.Single(types);
        Assert.Equal(2, types.First().Id);
        _storageServiceMock.Verify(x => x.SetItemAsync("activityTypes", It.IsAny<List<ActivityType>>()), Times.Once);
    }
}

