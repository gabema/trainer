namespace Trainer.Services;

using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.JSInterop;

internal sealed class ActiveActivityService : IActiveActivityService, IDisposable
{
    private const string StorageKey = "trainer_active_activities";

    private readonly Dictionary<int, DateTime> _active = new();
    private readonly IJSRuntime _js;
    private Timer? _fastTimer;
    private Timer? _slowTimer;

    public event Action? OnChanged;
    public event Action? OnTick;
    public event Action? OnSlowTick;

    public ActiveActivityService(IJSRuntime js)
    {
        _js = js;
    }

    public async Task InitializeAsync()
    {
        try
        {
            var json = await _js.InvokeAsync<string?>("localStorage.getItem", StorageKey).ConfigureAwait(false);
            if (string.IsNullOrEmpty(json)) return;

            var entries = JsonSerializer.Deserialize<List<StoredEntry>>(json);
            if (entries == null || entries.Count == 0) return;

            foreach (var entry in entries)
                _active[entry.Id] = entry.StartTime;

            EnsureTimersRunning();
            OnChanged?.Invoke();
        }
        catch (JSException)
        {
            // localStorage unavailable — start fresh
        }
        catch (JsonException)
        {
            // Corrupt stored data — clear it and start fresh
            await ClearStorageAsync().ConfigureAwait(false);
        }
    }

    public void Start(int activityId, DateTime startTime)
    {
        _active[activityId] = startTime;
        EnsureTimersRunning();
        OnChanged?.Invoke();
        _ = PersistAsync();
    }

    public void Finish(int activityId)
    {
        _active.Remove(activityId);
        if (_active.Count == 0)
            StopTimers();
        OnChanged?.Invoke();
        _ = PersistAsync();
    }

    public bool IsActive(int activityId) => _active.ContainsKey(activityId);

    public IReadOnlyDictionary<int, DateTime> GetAll() => _active;

    private async Task PersistAsync()
    {
        try
        {
            if (_active.Count == 0)
            {
                await ClearStorageAsync().ConfigureAwait(false);
                return;
            }

            var entries = _active
                .Select(kv => new StoredEntry(kv.Key, kv.Value))
                .ToList();
            var json = JsonSerializer.Serialize(entries);
            await _js.InvokeVoidAsync("localStorage.setItem", StorageKey, json).ConfigureAwait(false);
        }
        catch (JSException)
        {
            // Storage unavailable — continue without persistence
        }
    }

    private async Task ClearStorageAsync()
    {
        try
        {
            await _js.InvokeVoidAsync("localStorage.removeItem", StorageKey).ConfigureAwait(false);
        }
        catch (JSException)
        {
            // Ignore — storage may already be gone
        }
    }

    private void EnsureTimersRunning()
    {
        if (_fastTimer == null)
            _fastTimer = new Timer(_ => OnTick?.Invoke(), null, TimeSpan.FromSeconds(1), TimeSpan.FromSeconds(1));
        if (_slowTimer == null)
            _slowTimer = new Timer(_ => OnSlowTick?.Invoke(), null, TimeSpan.FromSeconds(30), TimeSpan.FromSeconds(30));
    }

    private void StopTimers()
    {
        _fastTimer?.Dispose();
        _fastTimer = null;
        _slowTimer?.Dispose();
        _slowTimer = null;
    }

    public void Dispose() => StopTimers();

    private sealed record StoredEntry(
        [property: JsonPropertyName("id")] int Id,
        [property: JsonPropertyName("startTime")] DateTime StartTime);
}
