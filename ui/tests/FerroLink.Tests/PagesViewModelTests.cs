using FerroLink.Core.Cli;
using FerroLink.Core.Models;
using FerroLink.Core.Services;
using FerroLink.Core.ViewModels;
using Xunit;

namespace FerroLink.Tests;

public sealed class PagesViewModelTests
{
    private static readonly Browser Brave = new("brave", "Brave", @"C:\b.exe", []);
    private static readonly LinkConfig Config = new("edge", true, [], [], ["ref"]);
    private readonly FakeCli _cli = new();

    private static AppStatus Status(bool http, bool https, Browser? fallback) =>
        new(new RegStatus(@"C:\fl.exe", http, https), @"C:\fl.exe", fallback);

    [Fact]
    public async Task BrowsersSaveSetsFallbackOnlyWhenChanged()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("rev1", Config)));
        _cli.StatusReply = FakeCli.Ok(Status(false, false, Brave));
        var vm = new BrowsersViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);
        await vm.LoadCommand.ExecuteAsync(null);
        Assert.Equal("edge", vm.DefaultBrowserId);
        Assert.Equal("brave", vm.FallbackBrowserId);

        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev2")));
        await vm.SaveCommand.ExecuteAsync(null);
        Assert.DoesNotContain("set-fallback:brave", _cli.Calls);

        vm.FallbackBrowserId = "edge";
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev3")));
        await vm.SaveCommand.ExecuteAsync(null);
        Assert.Contains("set-fallback:edge", _cli.Calls);
    }

    [Fact]
    public async Task TestLinkShowsResultAndIsDisabledWhenEmpty()
    {
        var vm = new TestLinkViewModel(_cli);
        Assert.False(vm.TestCommand.CanExecute(null));

        vm.Url = " https://a.com/?utm_source=x ";
        Assert.True(vm.TestCommand.CanExecute(null));

        _cli.TestUrlReply = FakeCli.Ok(new RouteResult("edge", "Edge", null, true, "https://a.com/"));
        await vm.TestCommand.ExecuteAsync(null);
        Assert.Equal("https://a.com/", vm.Result!.Url);
        Assert.Contains("test-url:https://a.com/?utm_source=x", _cli.Calls);
    }

    [Fact]
    public void ParseParamsSplitsTrimsAndDedupes()
    {
        Assert.Equal(new[] { "ref", "fbclid", "x" }, SettingsViewModel.ParseParams(" ref \n fbclid,REF\r\n\n x "));
    }

    [Fact]
    public async Task SettingsLoadAndSaveParams()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("rev1", Config)));
        _cli.StatusReply = FakeCli.Ok(Status(true, true, null));

        var vm = new SettingsViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);
        await vm.LoadCommand.ExecuteAsync(null);

        Assert.Equal("ref", vm.ExtraParams);
        Assert.True(vm.IsDefault);

        vm.ExtraParams = "ref, igshid";
        vm.StripTracking = false;

        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev2")));
        await vm.SaveCommand.ExecuteAsync(null);

        Assert.Equal(new[] { "ref", "igshid" }, _cli.LastSaved!.ExtraTrackingParams);
        Assert.False(_cli.LastSaved.StripTracking);
    }

    [Fact]
    public void IsDefaultRequiresHttpAndHttps()
    {
        var vm = new SettingsViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);

        vm.Status = Status(true, false, null);
        Assert.False(vm.IsDefault);

        vm.Status = Status(true, true, null);
        Assert.True(vm.IsDefault);
    }
}
