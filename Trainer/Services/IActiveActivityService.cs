namespace Trainer.Services;

internal interface IActiveActivityService
{
    /// <summary>Loads persisted active activities from localStorage. Call once on app startup.</summary>
    Task InitializeAsync();
    void Start(int activityId, DateTime startTime);
    void Finish(int activityId);
    bool IsActive(int activityId);
    IReadOnlyDictionary<int, DateTime> GetAll();
    event Action? OnChanged;
    event Action? OnTick;
    event Action? OnSlowTick;
}
