namespace Trainer.Tests.Services;

using Microsoft.JSInterop;
using Moq;
using Trainer.Services;

public class ActiveActivityServiceTests : IDisposable
{
    private readonly ActiveActivityService _sut = new(Mock.Of<IJSRuntime>());

    public void Dispose() => _sut.Dispose();

    [Fact]
    public void IsActive_BeforeStart_ReturnsFalse()
    {
        Assert.False(_sut.IsActive(1));
    }

    [Fact]
    public void Start_MakesActivityActive()
    {
        _sut.Start(1, DateTime.Now);
        Assert.True(_sut.IsActive(1));
    }

    [Fact]
    public void Start_RecordsSuppliedStartTime()
    {
        var when = new DateTime(2025, 6, 1, 10, 0, 0);
        _sut.Start(1, when);
        Assert.Equal(when, _sut.GetAll()[1]);
    }

    [Fact]
    public void Finish_RemovesActivity()
    {
        _sut.Start(1, DateTime.Now);
        _sut.Finish(1);
        Assert.False(_sut.IsActive(1));
    }

    [Fact]
    public void Finish_NonExistentActivity_DoesNotThrow()
    {
        _sut.Finish(999);
    }

    [Fact]
    public void GetAll_ReturnsAllActiveActivities()
    {
        _sut.Start(1, DateTime.Now);
        _sut.Start(2, DateTime.Now);
        var all = _sut.GetAll();
        Assert.Equal(2, all.Count);
        Assert.True(all.ContainsKey(1));
        Assert.True(all.ContainsKey(2));
    }

    [Fact]
    public void Start_RaisesOnChanged()
    {
        var fired = false;
        _sut.OnChanged += () => fired = true;
        _sut.Start(1, DateTime.Now);
        Assert.True(fired);
    }

    [Fact]
    public void Finish_RaisesOnChanged()
    {
        _sut.Start(1, DateTime.Now);
        var fired = false;
        _sut.OnChanged += () => fired = true;
        _sut.Finish(1);
        Assert.True(fired);
    }

    [Fact]
    public void Start_MultipleActivities_EachTrackedIndependently()
    {
        _sut.Start(10, DateTime.Now);
        _sut.Start(20, DateTime.Now);
        _sut.Finish(10);

        Assert.False(_sut.IsActive(10));
        Assert.True(_sut.IsActive(20));
    }
}
