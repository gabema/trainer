namespace Trainer.Tests.Helpers;

using Trainer.Helpers;

public class DecimalAmountTests
{
    // ── Format: raw integer -> decimal string ────────────────────────────────

    [Theory]
    [InlineData(0, 0, "0")]
    [InlineData(20, 0, "20")]
    [InlineData(125, 0, "125")]      // 0 places leaves the integer untouched
    public void Format_ZeroPlaces_ReturnsInteger(int amount, int places, string expected)
    {
        Assert.Equal(expected, DecimalAmount.Format(amount, places));
    }

    [Theory]
    [InlineData(125, 2, "1.25")]
    [InlineData(5, 2, "0.05")]       // left-pads with zeros
    [InlineData(0, 2, "0.00")]       // zero shape
    [InlineData(1250, 2, "12.50")]
    [InlineData(20, 2, "0.20")]      // the reinterpret case: 20 stored, shown at 2 places
    [InlineData(5, 3, "0.005")]
    [InlineData(1, 1, "0.1")]
    public void Format_WithPlaces_InsertsDecimalPoint(int amount, int places, string expected)
    {
        Assert.Equal(expected, DecimalAmount.Format(amount, places));
    }

    [Fact]
    public void Format_NegativeAmount_KeepsSign()
    {
        Assert.Equal("-1.25", DecimalAmount.Format(-125, 2));
    }

    // ── FormatDisplay: read-only form trims insignificant trailing zeros ──────

    [Theory]
    [InlineData(125, 2, "1.25")]     // no trailing zeros: unchanged
    [InlineData(120, 2, "1.2")]      // one trailing zero trimmed
    [InlineData(100, 2, "1")]        // all-zero fraction drops the decimal point
    [InlineData(50, 2, "0.5")]
    [InlineData(5, 2, "0.05")]       // significant zeros kept
    [InlineData(0, 2, "0")]
    [InlineData(200, 3, "0.2")]
    [InlineData(20, 0, "20")]        // 0 places unchanged
    [InlineData(-120, 2, "-1.2")]    // sign preserved
    public void FormatDisplay_TrimsTrailingZeros(int amount, int places, string expected)
    {
        Assert.Equal(expected, DecimalAmount.FormatDisplay(amount, places));
    }

    // ── ExtractDigits: models calculator accumulation / backspace / clear ─────

    [Fact]
    public void ExtractDigits_Empty_ReturnsNull()
    {
        // Only a field with no digit characters at all is treated as cleared.
        Assert.Null(DecimalAmount.ExtractDigits(""));
        Assert.Null(DecimalAmount.ExtractDigits(null));
        Assert.Null(DecimalAmount.ExtractDigits("."));
    }

    [Fact]
    public void ExtractDigits_AllZeros_ReturnsZero()
    {
        // Typed zeros are a real value (0), distinct from an empty/cleared field.
        Assert.Equal(0, DecimalAmount.ExtractDigits("0.00"));
    }

    [Theory]
    // Typing 1, 2, 5 into a 2-place field: each keystroke appends a digit and shifts left.
    [InlineData("0.01", 1)]
    [InlineData("0.12", 12)]
    [InlineData("1.25", 125)]
    public void ExtractDigits_Accumulation(string fieldText, int expected)
    {
        Assert.Equal(expected, DecimalAmount.ExtractDigits(fieldText));
    }

    [Fact]
    public void ExtractDigits_Backspace_DropsLastDigit()
    {
        // "1.25" backspaced to "1.2" -> digits "12"
        Assert.Equal(12, DecimalAmount.ExtractDigits("1.2"));
    }

    [Fact]
    public void ExtractDigits_CapsLength_ToAvoidOverflow()
    {
        // 12 digits supplied; only the first 9 are kept (well within Int32 range).
        Assert.Equal(123456789, DecimalAmount.ExtractDigits("123456789012"));
    }

    [Fact]
    public void RoundTrip_KeystrokeThenFormat_MatchesCalculatorBehavior()
    {
        // Simulate typing "5" onto "0.12" (field becomes "0.125") at 2 places.
        var accumulated = DecimalAmount.ExtractDigits("0.125");
        Assert.Equal(125, accumulated);
        Assert.Equal("1.25", DecimalAmount.Format(accumulated!.Value, 2));
    }
}
