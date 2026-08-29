namespace Trainer.Tests.Fixtures;

using System.Globalization;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Trainer.Models;
using Trainer.Serialization;
using Microsoft.JSInterop;
using Microsoft.JSInterop.Infrastructure;
using Moq;
using Trainer.Services;

/// <summary>
/// TEMPORARY. Generates golden fixtures from the C# implementation so the Rust port can be
/// asserted against real output rather than an assumed algorithm. Deleted by task 1.5 of the
/// rust-foundation change — any golden data the Rust port needs must be produced before then.
///
/// These tests only write files when TRAINER_GENERATE_FIXTURES is set. Otherwise they
/// no-op, so an ordinary `dotnet test` does not rewrite committed fixtures — which would
/// otherwise dirty the working tree, and produce differently named files per machine
/// (TimeZoneInfo.Local.Id is "PST8PDT" with no TZ set, "America/Los_Angeles" with it).
///
/// Run with:
///   TRAINER_GENERATE_FIXTURES=1 dotnet test --filter "FullyQualifiedName~GoldenFixtureGenerator"
///   TZ=Asia/Kolkata TRAINER_GENERATE_FIXTURES=1 dotnet test --filter "...GenerateTimestampFixture"
/// </summary>
public class GoldenFixtureGenerator
{
    private static bool GenerationRequested =>
        !string.IsNullOrEmpty(Environment.GetEnvironmentVariable("TRAINER_GENERATE_FIXTURES"));

    // Wide enough to cover many year boundaries in both directions, including the
    // 53-week years where WeekHelper's calendar-year + FirstFourDayWeek pairing
    // diverges from true ISO 8601 week-years.
    private const int FirstYear = 2010;
    private const int LastYear = 2040;

    private static string FixtureDirectory()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir != null && !File.Exists(Path.Combine(dir.FullName, "Trainer.sln")))
        {
            dir = dir.Parent;
        }

        if (dir == null)
        {
            throw new InvalidOperationException("Could not locate repository root (no Trainer.sln found above the test output directory).");
        }

        var fixtures = Path.Combine(dir.FullName, "trainer-rs", "tests", "fixtures");
        Directory.CreateDirectory(fixtures);
        return fixtures;
    }

    /// <summary>
    /// Task 1.1 — every day from FirstYear to LastYear paired with the week key
    /// WeekHelper produces for it.
    /// </summary>
    [Fact]
    public void GenerateWeekKeyFixture()
    {
        if (!GenerationRequested)
            return;

        var sb = new StringBuilder();
        sb.AppendLine("date,weekKey");

        var current = new DateTime(FirstYear, 1, 1);
        var end = new DateTime(LastYear, 12, 31);
        var rows = 0;

        while (current <= end)
        {
            sb.Append(current.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture))
              .Append(',')
              .AppendLine(WeekHelper.GetWeekKey(current));
            current = current.AddDays(1);
            rows++;
        }

        var path = Path.Combine(FixtureDirectory(), "week-keys.csv");
        File.WriteAllText(path, sb.ToString());

        Assert.True(rows > 11000, $"Expected a wide date span, generated only {rows} rows.");
    }

    /// <summary>
    /// Supports tasks 5.3 and 5.4. GetWeekStartDate / GetWeekEndDate must be verifiable
    /// after this generator is deleted, so their golden values are captured here.
    /// Includes the silent fallback behavior for week keys no date in the year maps to.
    /// </summary>
    [Fact]
    public void GenerateWeekBoundaryFixture()
    {
        if (!GenerationRequested)
            return;

        var sb = new StringBuilder();
        sb.AppendLine("weekKey,startDate,endDate");

        var seen = new SortedSet<string>(StringComparer.Ordinal);
        var current = new DateTime(FirstYear, 1, 1);
        var end = new DateTime(LastYear, 12, 31);

        while (current <= end)
        {
            seen.Add(WeekHelper.GetWeekKey(current));
            current = current.AddDays(1);
        }

        foreach (var weekKey in seen)
        {
            var start = WeekHelper.GetWeekStartDate(weekKey);
            var endDate = WeekHelper.GetWeekEndDate(weekKey);
            sb.Append(weekKey)
              .Append(',')
              .Append(start.ToString("yyyy-MM-ddTHH:mm:ss", CultureInfo.InvariantCulture))
              .Append(',')
              .AppendLine(endDate.ToString("yyyy-MM-ddTHH:mm:ss", CultureInfo.InvariantCulture));
        }

        var path = Path.Combine(FixtureDirectory(), "week-boundaries.csv");
        File.WriteAllText(path, sb.ToString());

        Assert.True(seen.Count > 1500, $"Expected many distinct week keys, generated only {seen.Count}.");
    }

    /// <summary>
    /// Task 1.3 — drives the REAL DateTimeConverter through the current process timezone,
    /// capturing both serializer configurations. The committed export only exercises
    /// Pacific hour-only offsets (-08/-07); this covers Z and non-zero-minute offsets.
    ///
    /// DateTimeConverter.Write reads TimeZoneInfo.Local, and .NET honors TZ on Unix, so:
    ///   TZ=America/Los_Angeles dotnet test --filter "...GenerateTimestampFixture"
    ///   TZ=Asia/Kolkata        dotnet test --filter "...GenerateTimestampFixture"
    ///   TZ=UTC                 dotnet test --filter "...GenerateTimestampFixture"
    ///
    /// Two option sets are captured because they differ in the shipping app and the Rust
    /// port therefore needs two serde configurations:
    ///   export  — ExportImportService: DefaultIgnoreCondition = WhenWritingNull
    ///   storage — IndexedDbStorageService: no ignore condition, so nulls are written
    /// </summary>
    [Fact]
    public void GenerateTimestampFixture()
    {
        if (!GenerationRequested)
            return;

        var exportOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
            Converters = { new DateTimeConverter() }
        };

        var storageOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            Converters = { new DateTimeConverter() }
        };

        // Local-kind instants spanning DST boundaries, year ends, and a leap day,
        // plus one Utc-kind instant to exercise the "Z" branch.
        var instants = new List<DateTime>
        {
            new(2026, 1, 1, 0, 0, 0, DateTimeKind.Local),
            new(2026, 1, 1, 8, 56, 44, DateTimeKind.Local),
            new(2026, 3, 8, 1, 59, 59, DateTimeKind.Local),   // just before US DST start
            new(2026, 3, 8, 3, 0, 0, DateTimeKind.Local),     // just after
            new(2026, 7, 4, 12, 30, 15, DateTimeKind.Local),
            new(2026, 11, 1, 1, 0, 0, DateTimeKind.Local),    // US DST end, ambiguous
            new(2026, 12, 31, 23, 59, 59, DateTimeKind.Local),
            new(2028, 2, 29, 6, 15, 0, DateTimeKind.Local),   // leap day
            new(2026, 6, 15, 10, 0, 0, DateTimeKind.Utc),     // -> "Z"
            new(2026, 6, 15, 10, 0, 0, DateTimeKind.Unspecified),
        };

        var activities = new List<Activity>();
        var id = 1;
        foreach (var when in instants)
        {
            // Alternate through the optional-field combinations so both option sets
            // are exercised against present, empty, and absent values.
            activities.Add(new Activity
            {
                Id = id,
                ActivityTypeId = 1,
                When = when,
                Amount = id * 3,
                Notes = id % 3 == 0 ? null : (id % 3 == 1 ? "" : "note text"),
                DurationSeconds = id % 2 == 0 ? null : id * 30,
                KnownLocationId = id % 4 == 0 ? null : id * 7,
            });
            id++;
        }

        var zone = TimeZoneInfo.Local.Id.Replace('/', '-');
        var dir = FixtureDirectory();

        File.WriteAllText(
            Path.Combine(dir, $"timestamps-export-{zone}.json"),
            JsonSerializer.Serialize(activities, exportOptions));

        File.WriteAllText(
            Path.Combine(dir, $"timestamps-storage-{zone}.json"),
            JsonSerializer.Serialize(activities, storageOptions));

        // Round-trip through the converter's own Read path, so the Rust port has a
        // recorded answer for what each emitted string parses back to.
        var reparsed = JsonSerializer.Deserialize<List<Activity>>(
            JsonSerializer.Serialize(activities, exportOptions), exportOptions)!;
        File.WriteAllText(
            Path.Combine(dir, $"timestamps-roundtrip-{zone}.json"),
            JsonSerializer.Serialize(reparsed, exportOptions));

        Assert.Equal(activities.Count, reparsed.Count);
    }

    /// <summary>
    /// Dumps exactly which characters System.Text.Json's default JavaScriptEncoder
    /// escapes. The shipping app sets no custom Encoder, so both serializer
    /// configurations use the default, which escapes far more than JSON requires
    /// (HTML-sensitive ASCII plus everything non-ASCII) as XSS defence-in-depth.
    /// serde_json escapes only the JSON minimum, so the port must reproduce this
    /// to keep exports byte-identical.
    /// </summary>
    [Fact]
    public void GenerateEscapingFixture()
    {
        if (!GenerationRequested)
            return;

        var options = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
            Converters = { new DateTimeConverter() }
        };

        var map = new SortedDictionary<string, string>(StringComparer.Ordinal);

        var probes = new List<int>();
        for (int c = 0; c < 128; c++)
            probes.Add(c);
        foreach (var c in new[] { 0x00A0, 0x00BD, 0x00E9, 0x20AC, 0x2019, 0x4E2D, 0x1F600 })
            probes.Add(c);

        foreach (var cp in probes)
        {
            var probe = char.ConvertFromUtf32(cp);
            var json = JsonSerializer.Serialize(probe, options);
            // Strip the surrounding quotes so the value is the encoded form alone.
            map[$"U+{cp:X4}"] = json.Substring(1, json.Length - 2);
        }

        // Emitted as JSON so the encoded forms survive without CSV quoting games.
        File.WriteAllText(
            Path.Combine(FixtureDirectory(), "json-escaping.json"),
            JsonSerializer.Serialize(map, options));
    }

    /// <summary>
    /// Task 1.4a. The captured profile had an empty localStorage, so neither the
    /// active-activity format nor the legacy migration has real-data backing.
    /// Both are therefore driven through the real C# code paths with a mocked
    /// IJSRuntime and the results recorded.
    ///
    /// ActiveActivityService persists with JsonSerializer.Serialize(entries) and
    /// NO options — so no DateTimeConverter. Its timestamps use System.Text.Json's
    /// default DateTime handling, which is a third wire format distinct from both
    /// the export and storage configurations.
    /// </summary>
    [Fact]
    public void GenerateActiveActivityFixture()
    {
        if (!GenerationRequested)
            return;

        var writes = new List<object?[]?>();
        var removes = new List<object?[]?>();

        var js = new Mock<IJSRuntime>();
        js.Setup(x => x.InvokeAsync<IJSVoidResult>(It.IsAny<string>(), It.IsAny<object?[]?>()))
          .Callback<string, object?[]?>((identifier, args) =>
          {
              if (identifier == "localStorage.setItem") writes.Add(args);
              if (identifier == "localStorage.removeItem") removes.Add(args);
          })
          .Returns(new ValueTask<IJSVoidResult>((IJSVoidResult)null!));

        using var service = new ActiveActivityService(js.Object);

        // Local-kind and Utc-kind start times, to expose how each is written.
        service.Start(1, new DateTime(2026, 8, 28, 15, 43, 21, DateTimeKind.Local));
        service.Start(7, new DateTime(2026, 6, 15, 10, 0, 0, DateTimeKind.Utc));
        service.Start(42, new DateTime(2026, 1, 1, 0, 0, 0, DateTimeKind.Unspecified));
        // The realistic case: DateTime.Now carries sub-second ticks, which the
        // default serializer renders as fractional seconds.
        service.Start(99, new DateTime(2026, 8, 28, 15, 43, 21, DateTimeKind.Local).AddTicks(1234567));
        service.Start(100, new DateTime(2026, 8, 28, 15, 43, 21, DateTimeKind.Local).AddTicks(1000000));

        var afterThree = writes.Count > 0 ? writes[^1]?[1] as string : null;

        service.Finish(7);
        var afterFinish = writes.Count > 0 ? writes[^1]?[1] as string : null;

        service.Finish(1);
        service.Finish(42);
        service.Finish(99);
        service.Finish(100);
        var removedWhenEmpty = removes.Count > 0;

        // Read path: feed a representative stored value back through InitializeAsync
        // and record what survives, since Read tolerance is as load-bearing as Write.
        var storedProbe = afterThree;
        var readBack = new Dictionary<string, string>(StringComparer.Ordinal);
        if (storedProbe != null)
        {
            var readJs = new Mock<IJSRuntime>();
            readJs.Setup(x => x.InvokeAsync<string?>("localStorage.getItem", It.IsAny<object?[]?>()))
                  .Returns(new ValueTask<string?>(storedProbe));
            readJs.Setup(x => x.InvokeAsync<IJSVoidResult>(It.IsAny<string>(), It.IsAny<object?[]?>()))
                  .Returns(new ValueTask<IJSVoidResult>((IJSVoidResult)null!));

            using var reader = new ActiveActivityService(readJs.Object);
            reader.InitializeAsync().GetAwaiter().GetResult();
            foreach (var kv in reader.GetAll())
            {
                readBack[kv.Key.ToString(CultureInfo.InvariantCulture)] =
                    $"{kv.Value:O} (Kind={kv.Value.Kind})";
            }
        }

        var report = new Dictionary<string, object?>
        {
            ["readBackFromStored"] = readBack,
            ["storageKey"] = "trainer_active_activities",
            ["serializerOptions"] = "JsonSerializer.Serialize(entries) with default options - no DateTimeConverter",
            ["afterThreeStarts"] = afterThree,
            ["afterOneFinish"] = afterFinish,
            ["removesKeyWhenEmpty"] = removedWhenEmpty,
            ["setItemCallCount"] = writes.Count,
            ["removeItemCallCount"] = removes.Count,
        };

        File.WriteAllText(
            Path.Combine(FixtureDirectory(), "active-activities.json"),
            JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
    }

    /// <summary>
    /// Task 1.4a, second half. Drives the real MigrateFromLocalStorageAsync with a
    /// mocked IJSRuntime and records what a pre-IndexedDB profile turns into, since
    /// the captured profile's localStorage was empty and this path has no real-data
    /// backing.
    /// </summary>
    [Fact]
    public void GenerateLegacyMigrationFixture()
    {
        if (!GenerationRequested)
            return;

        var storageOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            Converters = { new DateTimeConverter() }
        };

        // A flat list as the pre-IndexedDB app stored it, spanning a year boundary
        // so the migration has to split it across two week buckets.
        var legacyActivities = new List<Activity>
        {
            new() { Id = 1, ActivityTypeId = 1, When = new DateTime(2025, 12, 30, 8, 0, 0, DateTimeKind.Local), Amount = 16, Notes = "before new year" },
            new() { Id = 2, ActivityTypeId = 1, When = new DateTime(2026, 1, 2, 9, 30, 0, DateTimeKind.Local), Amount = 20, Notes = null },
            new() { Id = 3, ActivityTypeId = 2, When = new DateTime(2026, 1, 3, 18, 0, 0, DateTimeKind.Local), Amount = 5, Notes = "", DurationSeconds = 1800 },
            new() { Id = 4, ActivityTypeId = 2, When = new DateTime(2026, 2, 10, 7, 15, 0, DateTimeKind.Local), Amount = 3, KnownLocationId = 42 },
        };

        var legacyTypes = new List<ActivityType>
        {
            new() { Id = 1, Name = "Water", NetBenefit = NetBenefit.Positive, DailyAmount = 64, Unit = "oz" },
            new() { Id = 2, Name = "Run", NetBenefit = NetBenefit.Positive, Unit = "mi", DecimalPlaces = 2 },
        };

        var legacyActivitiesJson = JsonSerializer.Serialize(legacyActivities, storageOptions);
        var legacyTypesJson = JsonSerializer.Serialize(legacyTypes, storageOptions);

        var setItems = new List<(string Key, string Value)>();
        var removed = new List<string>();

        var js = new Mock<IJSRuntime>();
        js.Setup(x => x.InvokeAsync<string?>(It.IsAny<string>(), It.IsAny<object?[]?>()))
          .Returns((string identifier, object?[]? args) =>
          {
              var key = args is { Length: > 0 } ? args[0] as string : null;
              if (identifier == "localStorage.getItem" && key == "activities")
                  return new ValueTask<string?>(legacyActivitiesJson);
              if (identifier == "localStorage.getItem" && key == "activityTypes")
                  return new ValueTask<string?>(legacyTypesJson);
              return new ValueTask<string?>((string?)null);
          });
        js.Setup(x => x.InvokeAsync<IJSVoidResult>(It.IsAny<string>(), It.IsAny<object?[]?>()))
          .Callback<string, object?[]?>((identifier, args) =>
          {
              if (identifier == "indexedDbStorage.setItem" && args is { Length: > 1 })
                  setItems.Add((args[0] as string ?? "", args[1] as string ?? ""));
              if (identifier == "localStorage.removeItem" && args is { Length: > 0 })
                  removed.Add(args[0] as string ?? "");
          })
          .Returns(new ValueTask<IJSVoidResult>((IJSVoidResult)null!));

        using var service = new IndexedDbStorageService(js.Object);
        // Any public method triggers EnsureInitializedAsync, which runs the migration.
        service.ClearAsync().GetAwaiter().GetResult();

        var report = new Dictionary<string, object?>
        {
            ["legacyLocalStorage"] = new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["activities"] = legacyActivitiesJson,
                ["activityTypes"] = legacyTypesJson,
            },
            ["indexedDbWritesAfterMigration"] = setItems
                .ToDictionary(x => x.Key, x => x.Value, StringComparer.Ordinal),
            ["localStorageKeysRemoved"] = removed,
        };

        File.WriteAllText(
            Path.Combine(FixtureDirectory(), "legacy-migration.json"),
            JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
    }

    /// <summary>
    /// Pins how System.Text.Json writes doubles. KnownLocation latitude and
    /// longitude are doubles, and whole-valued ones are the case where .NET,
    /// serde_json and JSON.stringify can each disagree.
    /// </summary>
    [Fact]
    public void GenerateDoubleFormattingFixture()
    {
        if (!GenerationRequested)
            return;

        var options = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            Converters = { new DateTimeConverter() }
        };

        var probes = new[]
        {
            0.0, 1.0, 10.0, -20.0, 0.5, -0.5,
            37.4219983, -122.084, 21.111111, 19.9999999,
            1e21, 1e-7, 0.1, 1.0 / 3.0,
        };

        var map = new Dictionary<string, string>(StringComparer.Ordinal);
        foreach (var probe in probes)
        {
            var location = new KnownLocation { Id = 1, Name = "L", Latitude = probe, Longitude = 0.0 };
            var json = JsonSerializer.Serialize(location, options);
            map[probe.ToString("R", CultureInfo.InvariantCulture)] = json;
        }

        File.WriteAllText(
            Path.Combine(FixtureDirectory(), "double-formatting.json"),
            JsonSerializer.Serialize(map, new JsonSerializerOptions { WriteIndented = true }));
    }

    /// <summary>
    /// Pins CROSS-ZONE behavior: reads the Los Angeles fixture and re-serializes it under
    /// the current process timezone. DateTimeConverter.Read returns dto.DateTime (Kind
    /// Unspecified) for non-zero offsets, discarding the parsed offset, so Write then
    /// recomputes it from TimeZoneInfo.Local. Whether that shifts the instant is the
    /// question the Rust representation hinges on, so it is recorded rather than reasoned about.
    /// </summary>
    [Fact]
    public void GenerateCrossZoneFixture()
    {
        if (!GenerationRequested)
            return;

        var options = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
            Converters = { new DateTimeConverter() }
        };

        var dir = FixtureDirectory();
        var source = Path.Combine(dir, "timestamps-export-America-Los_Angeles.json");
        if (!File.Exists(source))
            return;

        var parsed = JsonSerializer.Deserialize<List<Activity>>(File.ReadAllText(source), options)!;
        var zone = TimeZoneInfo.Local.Id.Replace('/', '-');
        File.WriteAllText(
            Path.Combine(dir, $"timestamps-crosszone-from-LA-read-in-{zone}.json"),
            JsonSerializer.Serialize(parsed, options));
    }

    /// <summary>
    /// Pins GetWeekStartDate's fallback for week keys that no date in the year
    /// produces. The linear scan finds no match, leaves dateInWeek at January 1st,
    /// and returns that week's Monday. Task 5.3 requires reproducing this, and no
    /// other fixture reaches the branch.
    /// </summary>
    [Fact]
    public void GenerateUnmatchedWeekKeyFixture()
    {
        if (!GenerationRequested)
            return;

        var probes = new[] { "2026.99", "2026.00", "2010.60", "2026.54", "2013.53" };

        var sb = new StringBuilder();
        sb.AppendLine("weekKey,startDate,endDate,producedByAnyDateInYear");

        foreach (var weekKey in probes)
        {
            var year = int.Parse(weekKey.Split('.')[0], CultureInfo.InvariantCulture);
            var produced = false;
            for (var d = new DateTime(year, 1, 1); d <= new DateTime(year, 12, 31); d = d.AddDays(1))
            {
                if (WeekHelper.GetWeekKey(d) == weekKey) { produced = true; break; }
            }

            var start = WeekHelper.GetWeekStartDate(weekKey);
            var end = WeekHelper.GetWeekEndDate(weekKey);
            sb.Append(weekKey).Append(',')
              .Append(start.ToString("yyyy-MM-ddTHH:mm:ss", CultureInfo.InvariantCulture)).Append(',')
              .Append(end.ToString("yyyy-MM-ddTHH:mm:ss", CultureInfo.InvariantCulture)).Append(',')
              .AppendLine(produced ? "true" : "false");
        }

        File.WriteAllText(Path.Combine(FixtureDirectory(), "week-unmatched-keys.csv"), sb.ToString());
    }

    /// <summary>
    /// Records week keys where GetWeekStartDate does not round-trip: the scan finds a
    /// matching date, then walks back to that week's Monday, which lands in the previous
    /// year's bucket. Happens for the first week key of every year. These are the
    /// year-boundary cases the Rust port must reproduce.
    /// </summary>
    [Fact]
    public void GenerateWeekKeyAnomalyReport()
    {
        if (!GenerationRequested)
            return;

        var sb = new StringBuilder();
        sb.AppendLine("weekKey,sourceDate,resolvedStartDate,resolvedStartWeekKey,roundTrips");

        var current = new DateTime(FirstYear, 1, 1);
        var end = new DateTime(LastYear, 12, 31);
        var seen = new HashSet<string>(StringComparer.Ordinal);

        while (current <= end)
        {
            var weekKey = WeekHelper.GetWeekKey(current);
            if (seen.Add(weekKey))
            {
                var start = WeekHelper.GetWeekStartDate(weekKey);
                var resolved = WeekHelper.GetWeekKey(start);
                var roundTrips = string.Equals(resolved, weekKey, StringComparison.Ordinal);

                if (!roundTrips)
                {
                    sb.Append(weekKey)
                      .Append(',')
                      .Append(current.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture))
                      .Append(',')
                      .Append(start.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture))
                      .Append(',')
                      .Append(resolved)
                      .Append(',')
                      .AppendLine("false");
                }
            }
            current = current.AddDays(1);
        }

        var path = Path.Combine(FixtureDirectory(), "week-key-anomalies.csv");
        File.WriteAllText(path, sb.ToString());
    }
}
