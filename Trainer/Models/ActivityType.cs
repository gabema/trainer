namespace Trainer.Models;

public class ActivityType
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public NetBenefit NetBenefit { get; set; } = NetBenefit.None;
    public int? DailyAmount { get; set; }
    public int? WeeklyAmount { get; set; }
    public string? Unit { get; set; }
}

