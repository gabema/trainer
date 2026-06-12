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

    /// <summary>
    /// Number of decimal places amounts of this type are displayed and entered with.
    /// 0 (default) means whole numbers. Stored amounts are the raw integer value
    /// scaled by 10^DecimalPlaces (e.g. 125 displayed as 1.25 when DecimalPlaces is 2).
    /// </summary>
    public int DecimalPlaces { get; set; }
}

