namespace Trainer.Extensions;

/// <summary>
/// Extension methods for <see cref="string"/>.
/// </summary>
public static class StringExtensions
{
    /// <summary>
    /// Returns null if the string is null, empty, or contains only whitespace; otherwise returns the string unchanged.
    /// Use when saving to the model so empty/whitespace is stored as null.
    /// </summary>
    public static string? NullIfEmptyOrWhitespace(this string? value)
    {
        return string.IsNullOrWhiteSpace(value) ? null : value;
    }
}
