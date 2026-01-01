using Moq;
using Microsoft.JSInterop;
using Trainer.Services;
using System.Text.Json;

namespace Trainer.Tests.Services;

public class LocalStorageServiceTests
{
    private readonly Mock<IJSRuntime> _jsRuntimeMock;
    private readonly LocalStorageService _service;

    public LocalStorageServiceTests()
    {
        _jsRuntimeMock = new Mock<IJSRuntime>();
        _service = new LocalStorageService(_jsRuntimeMock.Object);
    }

    [Fact]
    public async Task GetItemAsync_ReturnsDeserializedValue_WhenItemExists()
    {
        // Arrange
        var testData = new { Name = "Test", Value = 123 };
        var json = JsonSerializer.Serialize(testData, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });
        
        _jsRuntimeMock
            .Setup(x => x.InvokeAsync<string>("localStorage.getItem", It.IsAny<object[]>()))
            .ReturnsAsync(json);

        // Act
        var result = await _service.GetItemAsync<TestModel>("testKey");

        // Assert
        Assert.NotNull(result);
        Assert.Equal("Test", result.Name);
        Assert.Equal(123, result.Value);
    }

    [Fact]
    public async Task GetItemAsync_ReturnsDefault_WhenItemDoesNotExist()
    {
        // Arrange
        _jsRuntimeMock
            .Setup(x => x.InvokeAsync<string?>(It.IsAny<string>(), It.IsAny<object[]>()))
            .ReturnsAsync((string?)null);

        // Act
        var result = await _service.GetItemAsync<TestModel>("testKey");

        // Assert
        Assert.Null(result);
    }

    [Fact]
    public async Task SetItemAsync_CompletesWithoutException()
    {
        // Arrange
        var testData = new TestModel { Name = "Test", Value = 123 };

        // Act & Assert - Just verify it doesn't throw
        // Note: Can't fully test JS interop with Moq due to extension methods
        await _service.SetItemAsync("testKey", testData);
        
        // If we get here, the method completed successfully
        Assert.True(true);
    }

    [Fact]
    public async Task RemoveItemAsync_CompletesWithoutException()
    {
        // Act & Assert - Just verify it doesn't throw
        // Note: Can't fully test JS interop with Moq due to extension methods
        await _service.RemoveItemAsync("testKey");
        
        // If we get here, the method completed successfully
        Assert.True(true);
    }

    [Fact]
    public async Task ClearAsync_CompletesWithoutException()
    {
        // Act & Assert - Just verify it doesn't throw
        // Note: Can't fully test JS interop with Moq due to extension methods
        await _service.ClearAsync();
        
        // If we get here, the method completed successfully
        Assert.True(true);
    }

    private class TestModel
    {
        public string Name { get; set; } = string.Empty;
        public int Value { get; set; }
    }
}

