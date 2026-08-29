namespace Trainer.Tests.Fixtures;

using System.Text.Json;
using Microsoft.JSInterop;
using Microsoft.JSInterop.Infrastructure;
using Moq;

/// <summary>
/// An IJSRuntime backing indexedDbStorage.* and localStorage.* with a plain
/// dictionary, so the real storage and service stack can run outside a browser.
///
/// Unlike GoldenFixtureGenerator this is NOT temporary: the cross-implementation
/// tests use it for as long as the C# project exists.
/// </summary>
internal static class InMemoryJsRuntime
{
    public static IJSRuntime Create(Dictionary<string, string> backing)
    {
        var js = new Mock<IJSRuntime>();

        // Nullable reference annotations are erased at runtime, so
        // InvokeAsync<string?> and InvokeAsync<string> are the SAME method.
        // Separate Setups silently override one another, so every string-typed
        // call must be dispatched from this one setup.
        js.Setup(x => x.InvokeAsync<string>(It.IsAny<string>(), It.IsAny<object?[]?>()))
          .Returns((string identifier, object?[]? args) =>
          {
              var first = args is { Length: > 0 } ? args[0] as string : null;

              if (identifier == "indexedDbStorage.getItem" && first != null)
                  return new ValueTask<string>(backing.TryGetValue(first, out var v) ? v : null!);

              if (identifier == "indexedDbStorage.getItems")
              {
                  var keys = first is null
                      ? new List<string>()
                      : JsonSerializer.Deserialize<List<string>>(first) ?? new List<string>();
                  var found = new Dictionary<string, JsonElement>(StringComparer.Ordinal);
                  foreach (var key in keys)
                  {
                      if (backing.TryGetValue(key, out var raw))
                          found[key] = JsonDocument.Parse(raw).RootElement.Clone();
                  }
                  return new ValueTask<string>(JsonSerializer.Serialize(found));
              }

              return new ValueTask<string>((string)null!);
          });

        js.Setup(x => x.InvokeAsync<string[]>(It.IsAny<string>(), It.IsAny<object?[]?>()))
          .Returns((string identifier, object?[]? args) =>
          {
              var prefix = args is { Length: > 0 } ? args[0] as string ?? "" : "";
              var keys = backing.Keys
                  .Where(k => k.StartsWith(prefix, StringComparison.Ordinal))
                  .ToArray();
              return new ValueTask<string[]>(keys);
          });

        js.Setup(x => x.InvokeAsync<IJSVoidResult>(It.IsAny<string>(), It.IsAny<object?[]?>()))
          .Callback<string, object?[]?>((identifier, args) =>
          {
              switch (identifier)
              {
                  case "indexedDbStorage.setItem" when args is { Length: > 1 }:
                      backing[args[0] as string ?? ""] = args[1] as string ?? "";
                      break;
                  case "indexedDbStorage.removeItem" when args is { Length: > 0 }:
                      backing.Remove(args[0] as string ?? "");
                      break;
                  case "indexedDbStorage.clear":
                      backing.Clear();
                      break;
              }
          })
          .Returns(new ValueTask<IJSVoidResult>((IJSVoidResult)null!));

        return js.Object;
    }

    /// <summary>Locates trainer-rs/tests/fixtures relative to the repository root.</summary>
    public static string FixturePath(string name)
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir != null && !File.Exists(Path.Combine(dir.FullName, "Trainer.sln")))
        {
            dir = dir.Parent;
        }

        if (dir == null)
        {
            throw new InvalidOperationException("Could not locate the repository root.");
        }

        return Path.Combine(dir.FullName, "trainer-rs", "tests", "fixtures", name);
    }
}
