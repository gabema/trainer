using Microsoft.JSInterop;
using System.Text.Json;

namespace Trainer.Services;

public class LocalStorageService : IStorageService
{
    private readonly IJSRuntime _jsRuntime;
    private readonly JsonSerializerOptions _jsonOptions;

    public LocalStorageService(IJSRuntime jsRuntime)
    {
        _jsRuntime = jsRuntime;
        _jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false
        };
    }

    public async Task<T?> GetItemAsync<T>(string key)
    {
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:21", message = "GetItemAsync entry", data = new { key = key, type = typeof(T).Name }, runId = "run1", hypothesisId = "C" });
        // #endregion
        try
        {
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:25", message = "Before localStorage.getItem JS interop", data = new { key = key }, runId = "run1", hypothesisId = "C" });
            // #endregion
            var json = await _jsRuntime.InvokeAsync<string>("localStorage.getItem", key);
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:26", message = "After localStorage.getItem JS interop", data = new { key = key, jsonLength = json?.Length ?? 0, hasJson = !string.IsNullOrEmpty(json) }, runId = "run1", hypothesisId = "C" });
            // #endregion
            if (string.IsNullOrEmpty(json))
                return default(T);

            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:29", message = "Before JSON deserialize", data = new { jsonLength = json?.Length ?? 0 }, runId = "run1", hypothesisId = "E" });
            // #endregion
            var result = JsonSerializer.Deserialize<T>(json, _jsonOptions);
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:30", message = "After JSON deserialize", data = new { success = result != null }, runId = "run1", hypothesisId = "E" });
            // #endregion
            return result;
        }
        catch (Exception ex)
        {
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:32", message = "GetItemAsync exception", data = new { exceptionType = ex.GetType().Name, exceptionMessage = ex.Message }, runId = "run1", hypothesisId = "C,E" });
            // #endregion
            return default(T);
        }
    }

    public async Task SetItemAsync<T>(string key, T value)
    {
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:37", message = "SetItemAsync entry", data = new { key = key, type = typeof(T).Name }, runId = "run1", hypothesisId = "C" });
        // #endregion
        var json = JsonSerializer.Serialize(value, _jsonOptions);
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:40", message = "Before localStorage.setItem JS interop", data = new { key = key, jsonLength = json?.Length ?? 0 }, runId = "run1", hypothesisId = "C" });
        // #endregion
        await _jsRuntime.InvokeVoidAsync("localStorage.setItem", key, json);
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:41", message = "After localStorage.setItem JS interop", data = new { key = key }, runId = "run1", hypothesisId = "C" });
        // #endregion
    }

    public async Task RemoveItemAsync(string key)
    {
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:43", message = "RemoveItemAsync entry", data = new { key = key }, runId = "run1", hypothesisId = "C" });
        // #endregion
        await _jsRuntime.InvokeVoidAsync("localStorage.removeItem", key);
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:46", message = "After localStorage.removeItem JS interop", data = new { key = key }, runId = "run1", hypothesisId = "C" });
        // #endregion
    }

    public async Task ClearAsync()
    {
        // #region agent log
        await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:48", message = "ClearAsync entry", data = new { }, runId = "run1", hypothesisId = "A" });
        // #endregion
        try
        {
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:51", message = "Before localStorage.clear JS interop", data = new { }, runId = "run1", hypothesisId = "A" });
            // #endregion
            await _jsRuntime.InvokeVoidAsync("localStorage.clear");
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:52", message = "After localStorage.clear JS interop - SUCCESS", data = new { }, runId = "run1", hypothesisId = "A" });
            // #endregion
        }
        catch (Exception ex)
        {
            // #region agent log
            await _jsRuntime.InvokeVoidAsync("debugLog", new { location = "LocalStorageService.cs:54", message = "ClearAsync exception", data = new { exceptionType = ex.GetType().Name, exceptionMessage = ex.Message, stackTrace = ex.StackTrace }, runId = "run1", hypothesisId = "A" });
            // #endregion
            throw;
        }
    }
}

