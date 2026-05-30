namespace Trainer.Tests.Services;

using Moq;
using Trainer.Models;
using Trainer.Services;

public class KnownLocationServiceTests
{
    private readonly Mock<IStorageService> _storageMock;
    private readonly KnownLocationService _service;

    public KnownLocationServiceTests()
    {
        _storageMock = new Mock<IStorageService>();
        _service = new KnownLocationService(_storageMock.Object);
    }

    private void SetupStorage(List<KnownLocation>? locations)
    {
        _storageMock
            .Setup(s => s.GetItemAsync<List<KnownLocation>>("knownLocations"))
            .ReturnsAsync(locations);
    }

    // ── GetAllAsync ──────────────────────────────────────────────────────────

    [Fact]
    public async Task GetAllAsync_ReturnsEmptyList_WhenNoneExist()
    {
        SetupStorage(null);
        var result = await _service.GetAllAsync();
        Assert.Empty(result);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsStoredLocations()
    {
        var locations = new List<KnownLocation>
        {
            new() { Id = 1, Name = "Home", Latitude = 37.77, Longitude = -122.41 },
            new() { Id = 2, Name = "Work", Latitude = 37.78, Longitude = -122.40 },
        };
        SetupStorage(locations);

        var result = await _service.GetAllAsync();

        Assert.Equal(2, result.Count);
    }

    // ── SaveAsync – new location ─────────────────────────────────────────────

    [Fact]
    public async Task SaveAsync_NewLocation_AssignsHashDerivedId()
    {
        SetupStorage(new List<KnownLocation>());
        _storageMock.Setup(s => s.SetItemAsync("knownLocations", It.IsAny<List<KnownLocation>>())).Returns(Task.CompletedTask);

        var location = new KnownLocation { Id = 0, Name = "Park", Latitude = 37.77, Longitude = -122.41 };
        var saved = await _service.SaveAsync(location);

        int expected = HashCode.Combine(37.77.GetHashCode(), (-122.41).GetHashCode());
        Assert.Equal(expected, saved.Id);
    }

    [Fact]
    public async Task SaveAsync_NewLocation_StoresInList()
    {
        List<KnownLocation>? captured = null;
        SetupStorage(new List<KnownLocation>());
        _storageMock
            .Setup(s => s.SetItemAsync("knownLocations", It.IsAny<List<KnownLocation>>()))
            .Callback<string, List<KnownLocation>>((_, list) => captured = list)
            .Returns(Task.CompletedTask);

        var location = new KnownLocation { Id = 0, Name = "Park", Latitude = 37.77, Longitude = -122.41 };
        await _service.SaveAsync(location);

        Assert.NotNull(captured);
        Assert.Single(captured!);
        Assert.Equal("Park", captured![0].Name);
    }

    [Fact]
    public async Task SaveAsync_HashCollision_IncrementsId()
    {
        double lat = 37.77, lon = -122.41;
        int hash = HashCode.Combine(lat.GetHashCode(), lon.GetHashCode());

        // Pre-populate with a location that has the exact hash id
        SetupStorage(new List<KnownLocation>
        {
            new() { Id = hash, Name = "Existing", Latitude = lat, Longitude = lon }
        });
        List<KnownLocation>? captured = null;
        _storageMock
            .Setup(s => s.SetItemAsync("knownLocations", It.IsAny<List<KnownLocation>>()))
            .Callback<string, List<KnownLocation>>((_, list) => captured = list)
            .Returns(Task.CompletedTask);

        var newLocation = new KnownLocation { Id = 0, Name = "Nearby", Latitude = lat, Longitude = lon };
        var saved = await _service.SaveAsync(newLocation);

        Assert.Equal(hash + 1, saved.Id);
    }

    // ── SaveAsync – update existing ──────────────────────────────────────────

    [Fact]
    public async Task SaveAsync_ExistingId_UpdatesRecord()
    {
        var existing = new KnownLocation { Id = 42, Name = "Home", Latitude = 37.77, Longitude = -122.41 };
        SetupStorage(new List<KnownLocation> { existing });
        List<KnownLocation>? captured = null;
        _storageMock
            .Setup(s => s.SetItemAsync("knownLocations", It.IsAny<List<KnownLocation>>()))
            .Callback<string, List<KnownLocation>>((_, list) => captured = list)
            .Returns(Task.CompletedTask);

        var updated = existing with { Name = "Home Office" };
        await _service.SaveAsync(updated);

        Assert.NotNull(captured);
        Assert.Single(captured!);
        Assert.Equal("Home Office", captured![0].Name);
    }

    // ── DeleteAsync ──────────────────────────────────────────────────────────

    [Fact]
    public async Task DeleteAsync_RemovesLocationById()
    {
        var locations = new List<KnownLocation>
        {
            new() { Id = 1, Name = "Home", Latitude = 37.77, Longitude = -122.41 },
            new() { Id = 2, Name = "Work", Latitude = 37.78, Longitude = -122.40 },
        };
        SetupStorage(locations);
        List<KnownLocation>? captured = null;
        _storageMock
            .Setup(s => s.SetItemAsync("knownLocations", It.IsAny<List<KnownLocation>>()))
            .Callback<string, List<KnownLocation>>((_, list) => captured = list)
            .Returns(Task.CompletedTask);

        await _service.DeleteAsync(1);

        Assert.NotNull(captured);
        Assert.Single(captured!);
        Assert.Equal(2, captured![0].Id);
    }

    // ── FindNearbyAsync ──────────────────────────────────────────────────────

    [Fact]
    public async Task FindNearbyAsync_ReturnsClosestWithin100m()
    {
        // 37.77493, -122.41942 is the test anchor; we put a location ~50 m away
        double anchorLat = 37.77493, anchorLon = -122.41942;
        var nearLocation = new KnownLocation { Id = 1, Name = "Near", Latitude = anchorLat + 0.0004, Longitude = anchorLon };
        SetupStorage(new List<KnownLocation> { nearLocation });

        var result = await _service.FindNearbyAsync(anchorLat, anchorLon);

        Assert.NotNull(result);
        Assert.Equal(1, result!.Id);
    }

    [Fact]
    public async Task FindNearbyAsync_ReturnsNull_WhenNoneWithin100m()
    {
        // Put a location ~500 m away
        double anchorLat = 37.77493, anchorLon = -122.41942;
        var farLocation = new KnownLocation { Id = 1, Name = "Far", Latitude = anchorLat + 0.005, Longitude = anchorLon };
        SetupStorage(new List<KnownLocation> { farLocation });

        var result = await _service.FindNearbyAsync(anchorLat, anchorLon);

        Assert.Null(result);
    }

    [Fact]
    public async Task FindNearbyAsync_ReturnsNull_WhenNoLocations()
    {
        SetupStorage(new List<KnownLocation>());
        var result = await _service.FindNearbyAsync(37.77, -122.41);
        Assert.Null(result);
    }

    // ── NextAutoNameAsync ────────────────────────────────────────────────────

    [Fact]
    public async Task NextAutoNameAsync_ReturnsNewLocation1_WhenNoneExist()
    {
        SetupStorage(new List<KnownLocation>());
        var name = await _service.NextAutoNameAsync();
        Assert.Equal("New Location 1", name);
    }

    [Fact]
    public async Task NextAutoNameAsync_ReturnsNewLocation2_WhenLocation1Exists()
    {
        SetupStorage(new List<KnownLocation>
        {
            new() { Id = 1, Name = "New Location 1", Latitude = 0, Longitude = 0 }
        });
        var name = await _service.NextAutoNameAsync();
        Assert.Equal("New Location 2", name);
    }

    [Fact]
    public async Task NextAutoNameAsync_SkipsGaps_ReturnsLowestAvailable()
    {
        SetupStorage(new List<KnownLocation>
        {
            new() { Id = 1, Name = "New Location 1", Latitude = 0, Longitude = 0 },
            new() { Id = 2, Name = "New Location 3", Latitude = 0, Longitude = 0 },
        });
        var name = await _service.NextAutoNameAsync();
        Assert.Equal("New Location 2", name);
    }
}
