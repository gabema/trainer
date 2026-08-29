namespace Trainer.Tests.Services;

using Trainer.Models;
using Trainer.Services;
using Trainer.Tests.Fixtures;

/// <summary>
/// Task 9.2 — the C# implementation must accept an export produced by the Rust
/// port. The fixture is written by the Rust test
/// `rust_produced_export_is_written_for_the_csharp_side`, so the two
/// implementations meet on a real file rather than on an assertion about one.
/// </summary>
public class RustInteropTests
{
    private static (ExportImportService Service, IActivityService Activities, IKnownLocationService Locations, Dictionary<string, string> Backing) Build()
    {
        var backing = new Dictionary<string, string>(StringComparer.Ordinal);
        var js = InMemoryJsRuntime.Create(backing);
        var storage = new IndexedDbStorageService(js);
        var activities = new ActivityService(storage);
        var locations = new KnownLocationService(storage);
        return (new ExportImportService(storage, activities, locations), activities, locations, backing);
    }

    private static string RustExport() =>
        File.ReadAllText(InMemoryJsRuntime.FixturePath("rust-export.json"));

    [Fact]
    public async Task ImportDataAsync_AcceptsRustProducedExport()
    {
        var (service, activityService, locationService, _) = Build();

        await service.ImportDataAsync(RustExport());

        var activities = await activityService.GetAllAsync();
        Assert.Equal(4, activities.Count);

        // Ids, amounts and the three notes states must all survive.
        Assert.Contains(activities, a => a.Notes == null);
        Assert.Contains(activities, a => a.Notes == string.Empty);
        Assert.Contains(activities, a => !string.IsNullOrEmpty(a.Notes));
        Assert.Contains(activities, a => a.DurationSeconds == 1800);

        var locations = await locationService.GetAllAsync();
        Assert.Equal(2, locations.Count);
        Assert.Contains(locations, l => l.Latitude == 10.0 && l.Longitude == -20.0);
    }

    [Fact]
    public async Task ImportDataAsync_RustExport_PreservesTimestampsAcrossTheYearBoundary()
    {
        var (service, activityService, _, backing) = Build();

        await service.ImportDataAsync(RustExport());

        // The Rust export groups by the same week keys the C# uses, including
        // the calendar-year boundary split.
        Assert.Contains("activities-2025.53", backing.Keys);
        Assert.Contains("activities-2026.01", backing.Keys);

        var activities = await activityService.GetAllAsync();
        Assert.Contains(activities, a => a.When.Year == 2025 && a.When.Month == 12);
        Assert.Contains(activities, a => a.When.Kind == DateTimeKind.Utc);
    }

    [Fact]
    public async Task ImportDataAsync_RustExport_PreservesActivityTypeDetail()
    {
        var (service, _, _, backing) = Build();

        await service.ImportDataAsync(RustExport());

        var typesJson = backing["activityTypes"];
        var types = System.Text.Json.JsonSerializer.Deserialize<List<ActivityType>>(
            typesJson,
            new System.Text.Json.JsonSerializerOptions
            {
                PropertyNamingPolicy = System.Text.Json.JsonNamingPolicy.CamelCase,
            })!;

        Assert.Equal(2, types.Count);
        Assert.Contains(types, t => t.IsPrivate && t.DecimalPlaces == 2);
        Assert.Contains(types, t => t.NetBenefit == NetBenefit.Positive);
        // Escaped characters must survive the round trip as real characters.
        Assert.Contains(types, t => t.Name.Contains('½'));
    }

    [Fact]
    public async Task RustAndCSharpExports_AgreeOnEverythingExceptTheRecomputedOffset()
    {
        // Importing the C#-produced fixture and re-exporting yields the same
        // bytes EXCEPT for each timestamp's UTC offset, which is why this
        // compares with offsets normalized away.
        //
        // DateTimeConverter.Read returns dto.DateTime for a non-zero offset,
        // discarding the parsed offset and leaving Kind Unspecified. Write then
        // recomputes it from TimeZoneInfo.Local. So a fixture generated in
        // Pacific re-exports as "-08" on a Pacific machine and as "Z" on a UTC
        // CI runner: the wall clock survives, the instant silently moves.
        //
        // Recorded in trainer-rs/tests/fixtures/timestamps-crosszone-*.json.
        // The Rust port does not share this behavior — it retains the parsed
        // offset — which is why the equivalent Rust test can compare raw bytes.
        var (service, _, _, _) = Build();
        var csharpExport = File.ReadAllText(InMemoryJsRuntime.FixturePath("csharp-export.json"));

        await service.ImportDataAsync(csharpExport);
        var reExported = await service.ExportDataAsync();

        Assert.Equal(Normalize(csharpExport), Normalize(reExported));
    }

    [Fact]
    public async Task ImportDataAsync_PreservesWallClockEvenWhenTheOffsetIsRecomputed()
    {
        // The portable half of the behavior above: whatever the machine
        // timezone, the wall clock a user sees must not move.
        var (service, activityService, _, _) = Build();
        await service.ImportDataAsync(
            File.ReadAllText(InMemoryJsRuntime.FixturePath("csharp-export.json")));

        var activities = await activityService.GetAllAsync();
        var earliest = activities.MinBy(a => a.When);

        Assert.NotNull(earliest);
        Assert.Equal(new DateTime(2025, 12, 30, 8, 0, 0), earliest!.When);
    }

    /// <summary>
    /// Strips the UTC offset from every timestamp, and blanks exportDate, which
    /// is stamped at export time.
    ///
    /// The offset appears in three forms and all must be handled: "Z", the
    /// hour-only negative form "-08", and a positive offset — which arrives as
    /// "\u002B05:30" rather than "+05:30", because JavaScriptEncoder.Default
    /// escapes "+". Missing that last case makes this test pass in Pacific and
    /// UTC while failing in every zone east of Greenwich.
    /// </summary>
    private static string Normalize(string json)
    {
        var withoutExportDate = System.Text.RegularExpressions.Regex.Replace(
            json, @"""exportDate"":""[^""]*""", @"""exportDate"":""""");

        return System.Text.RegularExpressions.Regex.Replace(
            withoutExportDate,
            @"(""when"":""\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})(Z|\\u002B\d{2}(:\d{2})?|-\d{2}(:\d{2})?)""",
            @"$1""");
    }
}
