using System.Globalization;

namespace Trainer.Services;

public static class WeekHelper
{
    private static readonly Calendar Calendar = CultureInfo.InvariantCulture.Calendar;
    private static readonly CalendarWeekRule WeekRule = CalendarWeekRule.FirstFourDayWeek;
    private static readonly DayOfWeek FirstDayOfWeek = DayOfWeek.Monday;

    /// <summary>
    /// Converts a DateTime to ISO 8601 week format (YYYY.WW)
    /// </summary>
    public static string GetWeekKey(DateTime dateTime)
    {
        var year = Calendar.GetYear(dateTime);
        var week = Calendar.GetWeekOfYear(dateTime, WeekRule, FirstDayOfWeek);
        return $"{year}.{week:D2}";
    }

    /// <summary>
    /// Converts a week key (YYYY.WW) back to the start date of that week
    /// Uses the same calculation as GetWeekOfYear to ensure consistency
    /// </summary>
    public static DateTime GetWeekStartDate(string weekKey)
    {
        var parts = weekKey.Split('.');
        if (parts.Length != 2 || !int.TryParse(parts[0], out var year) || !int.TryParse(parts[1], out var week))
        {
            throw new ArgumentException($"Invalid week key format: {weekKey}", nameof(weekKey));
        }

        // Find a date in the target week by iterating from January 1st
        // We'll find a date that has the matching week number, then get the Monday of that week
        var startDate = new DateTime(year, 1, 1);
        var endDate = new DateTime(year, 12, 31);
        
        // Find a date that belongs to the target week
        DateTime dateInWeek = startDate;
        for (var date = startDate; date <= endDate; date = date.AddDays(1))
        {
            var testWeekKey = GetWeekKey(date);
            if (testWeekKey == weekKey)
            {
                dateInWeek = date;
                break;
            }
        }
        
        // Get the Monday of the week containing this date
        var daysToMonday = ((int)dateInWeek.DayOfWeek - (int)DayOfWeek.Monday + 7) % 7;
        var weekStart = dateInWeek.AddDays(-daysToMonday).Date;
        
        return weekStart;
    }

    /// <summary>
    /// Gets the end date of a week (Sunday)
    /// </summary>
    public static DateTime GetWeekEndDate(string weekKey)
    {
        return GetWeekStartDate(weekKey).AddDays(6).Date.AddHours(23).AddMinutes(59).AddSeconds(59);
    }

    /// <summary>
    /// Gets all week keys between two dates (inclusive)
    /// </summary>
    public static IEnumerable<string> GetWeekKeysInRange(DateTime startDate, DateTime endDate)
    {
        var currentDate = startDate.Date;
        var end = endDate.Date;
        var seenWeeks = new HashSet<string>();

        // Iterate day-by-day to ensure we don't skip any weeks when dates are close together
        // (e.g., Dec 31, 2025 to Jan 5, 2026 spans two weeks: 2025.53 and 2026.01)
        while (currentDate <= end)
        {
            var weekKey = GetWeekKey(currentDate);
            if (seenWeeks.Add(weekKey))
            {
                yield return weekKey;
            }
            currentDate = currentDate.AddDays(1);
        }
    }

    /// <summary>
    /// Gets the storage key for a week (activities-YYYY.WW)
    /// </summary>
    public static string GetStorageKey(string weekKey)
    {
        return $"activities-{weekKey}";
    }

    /// <summary>
    /// Extracts the week key from a storage key (activities-YYYY.WW -> YYYY.WW)
    /// </summary>
    public static string ExtractWeekKey(string storageKey)
    {
        if (storageKey.StartsWith("activities-"))
        {
            return storageKey.Substring("activities-".Length);
        }
        throw new ArgumentException($"Invalid storage key format: {storageKey}", nameof(storageKey));
    }
}

