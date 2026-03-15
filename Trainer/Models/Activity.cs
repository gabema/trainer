namespace Trainer.Models;

using System.Text.Json.Serialization;
using Trainer.Serialization;

public record Activity
{
    public int Id { get; set; }
    public int ActivityTypeId { get; set; }
    public DateTime When { get; set; }
    public int Amount { get; set; }

    [JsonConverter(typeof(EmptyStringAsNullConverter))]
    public string Notes { get; set; } = string.Empty;
    public int? DurationSeconds { get; set; }
}

