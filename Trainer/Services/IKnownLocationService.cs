namespace Trainer.Services;

using Trainer.Models;

public interface IKnownLocationService
{
    Task<List<KnownLocation>> GetAllAsync();
    Task<KnownLocation> SaveAsync(KnownLocation location);
    Task DeleteAsync(int id);
    Task<KnownLocation?> FindNearbyAsync(double latitude, double longitude);
    Task<string> NextAutoNameAsync();
}
