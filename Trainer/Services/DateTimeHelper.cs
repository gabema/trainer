using Trainer.Models;

namespace Trainer.Services;

public static class DateTimeHelper
{
    /// <summary>
    /// Formats a DateTime relative to the current time according to the specified rules:
    /// - Less than 2 hours ago: "X minutes ago"
    /// - More than 2 hours ago but same day: time only (e.g., "3:42 pm")
    /// - Yesterday: "yesterday @ {time}" (e.g., "yesterday @ 2:25 am")
    /// - More than yesterday ago: short date and time (e.g., "Jan 10 @ 10:22 am")
    /// </summary>
    /// <param name="when">The DateTime to format</param>
    /// <param name="now">The current DateTime (defaults to DateTime.Now if not provided)</param>
    /// <returns>Formatted string representation of the DateTime</returns>
    public static string FormatWhenDateTime(DateTime when, DateTime? now = null)
    {
        var currentTime = now ?? DateTime.Now;
        var timeDiff = currentTime - when;

        // Handle future dates - always show short date and time format
        if (when > currentTime)
        {
            return $"{when.ToString("MMM d")} @ {when.ToString("h:mm tt").ToLower()}";
        }

        // Less than 2 hours ago: show "X minutes ago"
        if (timeDiff.TotalMinutes < 120)
        {
            var minutes = (int)timeDiff.TotalMinutes;
            return minutes <= 1 ? "1 minute ago" : $"{minutes} minutes ago";
        }

        // Check if same day
        if (when.Date == currentTime.Date)
        {
            // More than 2 hours ago but same day: show time only
            return when.ToString("h:mm tt").ToLower();
        }

        // Check if yesterday
        var yesterday = currentTime.Date.AddDays(-1);
        if (when.Date == yesterday)
        {
            // Yesterday: show "yesterday @ {time}"
            return $"yesterday @ {when.ToString("h:mm tt").ToLower()}";
        }

        // More than yesterday ago: show short date and time
        return $"{when.ToString("MMM d")} @ {when.ToString("h:mm tt").ToLower()}";
    }

    public static (DateTime StartDate, DateTime EndDate) GetDateRange(DurationOption duration, DateTime? now = null)
    {
        var currentTime = now ?? DateTime.Now;
        
        switch (duration)
        {
            case DurationOption.Last24Hours:
                var start24h = currentTime.AddHours(-24);
                return (start24h, currentTime);
                
            case DurationOption.Last7Days:
                var start7d = currentTime.AddDays(-7);
                return (start7d, currentTime);
                
            case DurationOption.Last4Weeks:
                var start4w = currentTime.AddDays(-28);
                return (start4w, currentTime);
                
            case DurationOption.Week:
                // Get the Monday of the week containing the current date
                var daysToMonday = ((int)currentTime.DayOfWeek - (int)DayOfWeek.Monday + 7) % 7;
                var weekStart = currentTime.AddDays(-daysToMonday).Date;
                var weekEnd = weekStart.AddDays(6).Date.AddHours(23).AddMinutes(59).AddSeconds(59);
                return (weekStart, weekEnd);
                
            default:
                return (DateTime.MinValue, DateTime.MaxValue);
        }
    }
}
