namespace Trainer.Tests.Fixtures;

using System.Globalization;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Trainer.Models;
using Trainer.Serialization;
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
