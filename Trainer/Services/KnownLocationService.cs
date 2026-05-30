namespace Trainer.Services;

using Trainer.Models;

internal class KnownLocationService(IStorageService storageService) : IKnownLocationService
{
    private readonly IStorageService _storageService = storageService;
    private const string StorageKey = "knownLocations";
    private const double NearbyThresholdMetres = 100.0;

    public async Task<List<KnownLocation>> GetAllAsync()
    {
        return await _storageService.GetItemAsync<List<KnownLocation>>(StorageKey).ConfigureAwait(false)
               ?? new List<KnownLocation>();
    }

    public async Task<KnownLocation> SaveAsync(KnownLocation location)
    {
        ArgumentNullException.ThrowIfNull(location);
        var locations = await GetAllAsync().ConfigureAwait(false);

        if (location.Id == 0)
        {
            location = location with { Id = AssignId(location.Latitude, location.Longitude, locations) };
            locations.Add(location);
        }
        else
        {
            var index = locations.FindIndex(l => l.Id == location.Id);
            if (index >= 0)
                locations[index] = location;
            else
                locations.Add(location);
        }

        await _storageService.SetItemAsync(StorageKey, locations).ConfigureAwait(false);
        return location;
    }

    public async Task DeleteAsync(int id)
    {
        var locations = await GetAllAsync().ConfigureAwait(false);
        locations.RemoveAll(l => l.Id == id);
        await _storageService.SetItemAsync(StorageKey, locations).ConfigureAwait(false);
    }

    public async Task<KnownLocation?> FindNearbyAsync(double latitude, double longitude)
    {
        var locations = await GetAllAsync().ConfigureAwait(false);
        KnownLocation? closest = null;
        double closestDistance = double.MaxValue;

        foreach (var loc in locations)
        {
            var distance = HaversineMetres(latitude, longitude, loc.Latitude, loc.Longitude);
            if (distance < NearbyThresholdMetres && distance < closestDistance)
            {
                closest = loc;
                closestDistance = distance;
            }
        }

        return closest;
    }

    // Computes the next sequential "New Location {n}" name not yet used.
    public async Task<string> NextAutoNameAsync()
    {
        var locations = await GetAllAsync().ConfigureAwait(false);
        var usedNumbers = new HashSet<int>();
        foreach (var loc in locations)
        {
            if (loc.Name.StartsWith("New Location ", StringComparison.Ordinal) &&
                int.TryParse(loc.Name["New Location ".Length..], out var n))
            {
                usedNumbers.Add(n);
            }
        }

        int next = 1;
        while (usedNumbers.Contains(next))
            next++;

        return $"New Location {next}";
    }

    private static int AssignId(double latitude, double longitude, List<KnownLocation> existing)
    {
        var existingIds = new HashSet<int>(existing.Select(l => l.Id));
        int candidate = HashCode.Combine(latitude.GetHashCode(), longitude.GetHashCode());
        while (existingIds.Contains(candidate))
            candidate++;
        return candidate;
    }

    private static double HaversineMetres(double lat1, double lon1, double lat2, double lon2)
    {
        const double R = 6_371_000.0;
        var dLat = ToRad(lat2 - lat1);
        var dLon = ToRad(lon2 - lon1);
        var a = Math.Sin(dLat / 2) * Math.Sin(dLat / 2)
              + Math.Cos(ToRad(lat1)) * Math.Cos(ToRad(lat2))
              * Math.Sin(dLon / 2) * Math.Sin(dLon / 2);
        return R * 2 * Math.Atan2(Math.Sqrt(a), Math.Sqrt(1 - a));
    }

    private static double ToRad(double degrees) => degrees * Math.PI / 180.0;
}
