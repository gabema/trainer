namespace Trainer.Tests.Helpers;

using Trainer.Helpers;

public class DecimalPlacesWarningTests
{
    [Fact]
    public void ShouldWarn_ChangedAndHasActivities_True()
    {
        Assert.True(DecimalPlacesWarning.ShouldWarn(savedDecimalPlaces: 0, currentDecimalPlaces: 2, activityCount: 5));
    }

    [Fact]
    public void ShouldWarn_ChangedButNoActivities_False()
    {
        Assert.False(DecimalPlacesWarning.ShouldWarn(savedDecimalPlaces: 0, currentDecimalPlaces: 2, activityCount: 0));
    }

    [Fact]
    public void ShouldWarn_HasActivitiesButUnchanged_False()
    {
        Assert.False(DecimalPlacesWarning.ShouldWarn(savedDecimalPlaces: 2, currentDecimalPlaces: 2, activityCount: 5));
    }

    [Fact]
    public void ShouldWarn_NewType_NoActivitiesNoChange_False()
    {
        Assert.False(DecimalPlacesWarning.ShouldWarn(savedDecimalPlaces: 0, currentDecimalPlaces: 0, activityCount: 0));
    }
}
