using Switchman.Core.Cli;
using Switchman.Core.Models;
using Switchman.Core.Services;
using Switchman.Core.ViewModels;
using Xunit;

namespace Switchman.Tests;

public sealed class RulesViewModelTests
{
    private static readonly Browser Chrome = new("chrome", "Google Chrome", @"C:\c.exe", []);
    private static readonly LinkConfig TwoRules = new("edge", true, [new Rule("r1", "github.com", "chrome"), new Rule("r7", "x.com", "edge")], [], []);
    private readonly FakeCli _cli = new();

    private RulesViewModel Loaded()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("rev1", TwoRules)));
        _cli.DetectReply = FakeCli.Ok<IReadOnlyList<Browser>>([Chrome]);
        return new RulesViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);
    }

    [Fact]
    public async Task LoadFillsRulesAndBrowsers()
    {
        var vm = Loaded();
        await vm.LoadCommand.ExecuteAsync(null);

        Assert.Equal(2, vm.Rules.Count);
        Assert.Equal("github.com", vm.Rules[0].Pattern);
        Assert.Single(vm.Browsers);
    }

    [Fact]
    public async Task AddRuleUsesNextIdAndDefaultBrowser()
    {
        var vm = Loaded();
        await vm.LoadCommand.ExecuteAsync(null);

        vm.AddRuleCommand.Execute(null);

        Assert.Equal("r8", vm.Rules[2].Id);
        Assert.Equal("edge", vm.Rules[2].BrowserId);
    }

    [Fact]
    public async Task SaveSendsEditedRulesTrimmed()
    {
        var vm = Loaded();
        await vm.LoadCommand.ExecuteAsync(null);

        vm.Rules[0].Pattern = " gitlab.com ";
        vm.RemoveRuleCommand.Execute(vm.Rules[1]);

        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev2")));
        await vm.SaveCommand.ExecuteAsync(null);

        var saved = Assert.Single(_cli.LastSaved!.Rules);
        Assert.Equal("gitlab.com", saved.Pattern);
        Assert.Equal("edge", _cli.LastSaved.DefaultBrowserId);
        Assert.Equal("Guardado", vm.Message);
        Assert.Contains("set-config:rev1", _cli.Calls);
    }

    [Fact]
    public async Task FailedSaveShowsCliError()
    {
        var vm = Loaded();
        await vm.LoadCommand.ExecuteAsync(null);

        _cli.SetConfigReplies.Enqueue(FakeCli.Fail<SaveResult>(CliExit.Error, "patrón inválido: x"));
        await vm.SaveCommand.ExecuteAsync(null);

        Assert.Equal("patrón inválido: x", vm.Message);
    }

    [Fact]
    public async Task CorruptConfigShowsErrorAndEmptyList()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Fail<VersionedConfig>(CliExit.Corrupt, "config.json corrupto"));
        var vm = new RulesViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);

        await vm.LoadCommand.ExecuteAsync(null);
        Assert.Empty(vm.Rules);
        Assert.Equal("config.json corrupto", vm.Message);
    }

    [Fact]
    public async Task FirstSaveUsesFirstDetectedBrowserAsDefault()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("none", LinkConfig.Empty)));
        _cli.DetectReply = FakeCli.Ok<IReadOnlyList<Browser>>([Chrome]);
        var vm = new RulesViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);
        await vm.LoadCommand.ExecuteAsync(null);
        vm.AddRuleCommand.Execute(null);
        vm.Rules[0].Pattern = "github.com";
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev1")));

        await vm.SaveCommand.ExecuteAsync(null);

        Assert.Equal("chrome", _cli.LastSaved!.DefaultBrowserId);
        Assert.Contains("Google Chrome", vm.Message, StringComparison.Ordinal);
    }
}
