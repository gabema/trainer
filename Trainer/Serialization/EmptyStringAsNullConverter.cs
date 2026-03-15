namespace Trainer.Serialization;

using System.Text.Json;
using System.Text.Json.Serialization;

/// <summary>
/// Serializes empty string as JSON null (so it can be omitted with WhenWritingNull).
/// Deserializes JSON null or missing value as empty string.
/// </summary>
internal sealed class EmptyStringAsNullConverter : JsonConverter<string>
{
    public override string Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType == JsonTokenType.Null)
            return string.Empty;
        return reader.GetString() ?? string.Empty;
    }

    public override void Write(Utf8JsonWriter writer, string value, JsonSerializerOptions options)
    {
        if (string.IsNullOrEmpty(value))
            writer.WriteNullValue();
        else
            writer.WriteStringValue(value);
    }
}
