namespace Trainer.Tests.Helpers;

using Trainer.Helpers;

public class DurationInputTests
{
    [Theory]
    [InlineData("0:30", 30)]    // sub-minute M:SS is accepted (issue #83)
    [InlineData("5:30", 330)]   // minutes and seconds
    [InlineData("0:05", 5)]     // leading-zero seconds component
    [InlineData("20", 1200)]    // plain minutes
    [InlineData("0", 0)]        // zero minutes
    public void TryParse_ValidInput_ReturnsExpectedSeconds(string input, int expectedSeconds)
    {
        var ok = DurationInput.TryParse(input, out var seconds, out var error);

        Assert.True(ok);
        Assert.Null(error);
        Assert.Equal(expectedSeconds, seconds);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("   ")]
    public void TryParse_BlankInput_MeansNoDuration(string? input)
    {
        var ok = DurationInput.TryParse(input, out var seconds, out var error);

        Assert.True(ok);
        Assert.Null(error);
        Assert.Null(seconds);
    }

    [Theory]
    [InlineData("5:60")]    // seconds out of range
    [InlineData("abc")]     // non-numeric
    [InlineData("-1")]      // negative minutes
    [InlineData("1:2:3")]   // too many parts
    [InlineData("1000")]    // minutes too large
    public void TryParse_InvalidInput_ReturnsError(string input)
    {
        var ok = DurationInput.TryParse(input, out var seconds, out var error);

        Assert.False(ok);
        Assert.NotNull(error);
        Assert.Null(seconds);
    }
}
