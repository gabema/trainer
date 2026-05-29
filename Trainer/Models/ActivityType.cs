namespace Trainer.Models;

public record ActivityType
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public NetBenefit NetBenefit { get; set; } = NetBenefit.Neutral;
    public int? DailyAmount { get; set; }
    public int? WeeklyAmount { get; set; }
    public string? Unit { get; set; }
    public bool IsPrivate { get; set; }
}

