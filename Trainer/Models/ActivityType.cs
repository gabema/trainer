namespace Trainer.Models;

public class ActivityType
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public NetBenefit NetBenefit { get; set; } = NetBenefit.None;
    public int? DailyAmount { get; set; }
    public int? MonthlyAmount { get; set; }
}

