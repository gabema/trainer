namespace Trainer.Models;

public record Activity
{
    public int Id { get; set; }
    public int ActivityTypeId { get; set; }
    public DateTime When { get; set; }
    public int Amount { get; set; }
    public string? Notes { get; set; }
    public int? DurationSeconds { get; set; }
    public double? Latitude { get; set; }
    public double? Longitude { get; set; }
    public int? KnownLocationId { get; set; }
}

