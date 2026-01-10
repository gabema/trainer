namespace Trainer.Models;

public record Activity
{
    public int Id { get; set; }
    public int ActivityTypeId { get; set; }
    public DateTime When { get; set; }
    public int Amount { get; set; }
    public string Notes { get; set; } = string.Empty;
}

