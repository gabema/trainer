namespace Trainer.Serialization;

using System.Globalization;
using System.Text.Json;
using System.Text.Json.Serialization;

/// <summary>
/// Serializes DateTime with seconds precision (no milliseconds) and timezone offset as hour-only
/// when the minute component is zero (e.g. "-05" instead of "-05:00", "+05:30" when non-zero).
/// </summary>
internal sealed class DateTimeConverter : JsonConverter<DateTime>
{
    private const string FormatSeconds = "yyyy-MM-ddTHH:mm:ss";

    public override DateTime Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType == JsonTokenType.Null)
            return default;

        var s = reader.GetString();
        if (string.IsNullOrEmpty(s))
            return default;

        var dto = DateTimeOffset.Parse(s, CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind);
        return dto.Offset == TimeSpan.Zero ? dto.UtcDateTime : dto.DateTime;
    }

    public override void Write(Utf8JsonWriter writer, DateTime value, JsonSerializerOptions options)
    {
        string formatted;
        if (value.Kind == DateTimeKind.Utc)
        {
            formatted = value.ToString(FormatSeconds, CultureInfo.InvariantCulture) + "Z";
        }
        else
        {
            var offset = value.Kind == DateTimeKind.Local
                ? TimeZoneInfo.Local.GetUtcOffset(value)
                : TimeZoneInfo.Local.GetUtcOffset(DateTime.SpecifyKind(value, DateTimeKind.Local));
            var dto = new DateTimeOffset(value, offset);
            formatted = dto.ToString(FormatSeconds, CultureInfo.InvariantCulture) + FormatOffset(offset);
        }

        writer.WriteStringValue(formatted);
    }

    private static string FormatOffset(TimeSpan offset)
    {
        if (offset == TimeSpan.Zero)
            return "Z";

        int totalMinutes = (int)offset.TotalMinutes;
        int hours = totalMinutes / 60;
        int minutes = Math.Abs(totalMinutes % 60);
        string sign = totalMinutes >= 0 ? "+" : "-";
        string h = Math.Abs(hours).ToString("00", CultureInfo.InvariantCulture);
        return minutes == 0 ? $"{sign}{h}" : $"{sign}{h}:{minutes:D2}";
    }
}
