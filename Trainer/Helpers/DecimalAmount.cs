namespace Trainer.Helpers;

using System.Globalization;

/// <summary>
/// Converts between an activity's raw integer <c>Amount</c> and its displayed decimal
/// form. The stored integer is the value scaled by 10^decimalPlaces (e.g. 125 with
/// 2 places displays as "1.25"). The calculator-style input in <c>decimal-input.js</c>
/// mirrors these same rules; keep the two in sync.
/// </summary>
public static class DecimalAmount
{
    private const int MaxDigits = 9;

    /// <summary>
    /// Formats a raw integer amount as its decimal string for the given precision.
    /// With <paramref name="decimalPlaces"/> of 0 the value is returned unchanged
    /// (e.g. 20 → "20"); otherwise the decimal point is inserted that many digits
    /// from the right, left-padded with zeros (e.g. 5 @ 2 → "0.05", 125 @ 2 → "1.25").
    /// </summary>
    public static string Format(int amount, int decimalPlaces)
    {
        if (decimalPlaces <= 0)
        {
            return amount.ToString(CultureInfo.InvariantCulture);
        }

        var negative = amount < 0;
        var digits = Math.Abs((long)amount)
            .ToString(CultureInfo.InvariantCulture)
            .PadLeft(decimalPlaces + 1, '0');
        var dot = digits.Length - decimalPlaces;
        var result = $"{digits[..dot]}.{digits[dot..]}";
        return negative ? $"-{result}" : result;
    }

    /// <summary>
    /// Formats a raw integer amount for read-only display, dropping insignificant
    /// trailing zeros in the fractional part (and the decimal point when nothing
    /// remains): 120 @ 2 → "1.2", 100 @ 2 → "1", 125 @ 2 → "1.25", 5 @ 2 → "0.05".
    /// Use <see cref="Format"/> (fixed precision) for entry fields instead.
    /// </summary>
    public static string FormatDisplay(int amount, int decimalPlaces)
    {
        var formatted = Format(amount, decimalPlaces);
        if (decimalPlaces <= 0)
        {
            return formatted;
        }

        return formatted.TrimEnd('0').TrimEnd('.');
    }

    /// <summary>
    /// Extracts the raw integer accumulator from arbitrary input text by keeping only
    /// the digit characters (capped to a safe length). This is the transformation
    /// behind calculator-style entry: typing a digit appends it and shifts the value
    /// left, backspace drops the last digit. Returns null when no digits are present.
    /// </summary>
    public static int? ExtractDigits(string? input)
    {
        if (string.IsNullOrEmpty(input))
        {
            return null;
        }

        Span<char> buffer = stackalloc char[MaxDigits];
        var length = 0;
        foreach (var c in input)
        {
            if (char.IsDigit(c))
            {
                if (length == MaxDigits)
                {
                    break;
                }

                buffer[length++] = c;
            }
        }

        return length == 0 ? null : int.Parse(buffer[..length], CultureInfo.InvariantCulture);
    }
}
