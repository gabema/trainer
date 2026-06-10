namespace Trainer.Helpers;

/// <summary>
/// Parses the activity Duration field, which accepts either a whole number of
/// minutes (e.g. "20") or a colon-separated M:SS value (e.g. "5:30", "0:30").
/// </summary>
public static class DurationInput
{
    public static bool TryParse(string? input, out int? durationSeconds, out string? error)
    {
        durationSeconds = null;
        error = null;

        if (string.IsNullOrWhiteSpace(input))
        {
            return true;
        }

        input = input.Trim();
        var parts = input.Split(':');

        if (parts.Length == 1)
        {
            // Single integer = minutes, e.g., "20" means 20 minutes
            if (!int.TryParse(parts[0], out var minutesOnly))
            {
                error = "Duration must be a number of minutes or in M:SS format.";
                return false;
            }

            if (minutesOnly < 0)
            {
                error = "Duration cannot be negative.";
                return false;
            }

            if (minutesOnly > 999)
            {
                error = "Minutes must be less than 1000.";
                return false;
            }

            durationSeconds = minutesOnly * 60;
            return true;
        }

        if (parts.Length != 2)
        {
            error = "Duration must be a number of minutes (e.g., 20) or in M:SS format (e.g., 5:30).";
            return false;
        }

        if (!int.TryParse(parts[0], out var minutes) || !int.TryParse(parts[1], out var seconds))
        {
            error = "Minutes and seconds must be numeric.";
            return false;
        }

        if (minutes < 0 || seconds < 0)
        {
            error = "Duration cannot be negative.";
            return false;
        }

        if (seconds >= 60)
        {
            error = "Seconds must be between 00 and 59.";
            return false;
        }

        if (minutes > 999)
        {
            error = "Minutes must be less than 1000.";
            return false;
        }

        durationSeconds = (minutes * 60) + seconds;
        return true;
    }
}
