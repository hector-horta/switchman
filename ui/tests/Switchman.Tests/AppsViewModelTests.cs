using Switchman.Core.Cli;
using Switchman.Core.Models;
using Switchman.Core.Services;
using Switchman.Core.ViewModels;
using Xunit;

namespace Switchman.Tests;

public sealed class AppsViewModelTests
{
    private static readonly AppsInfo Info = new(
        [
            new CatalogAppInfo("whatsapp", "WhatsApp", "whatsapp", ["wa.me", "whatsapp.com"], true, true),
            new CatalogAppInfo("zoom", "Zoom", "zoommtg", ["zoom.us"], false, true),
        ],
        [],
        ["bitwarden", "steam"]);

    private readonly FakeCli _cli = new();

    private async Task<AppsViewModel> LoadedAsync()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("rev1", new LinkConfig("edge", true, [], [], [], ["whatsapp", "zoom"], []))));
        _cli.AppsReply = FakeCli.Ok(Info);
        var vm = new AppsViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);
        await vm.LoadCommand.ExecuteAsync(null);
        return vm;
    }

    [Fact]
    public async Task LoadShowsOnlyInstalledAppsWithTheirState()
    {
        var vm = await LoadedAsync();
        var app = Assert.Single(vm.Apps);
        Assert.Equal("whatsapp", app.Id);
        Assert.True(app.Enabled);
        Assert.Equal("wa.me, whatsapp.com", app.Domains);
        Assert.Contains("Zoom", vm.NotInstalled, StringComparison.Ordinal);
        Assert.Equal(new[] { "bitwarden", "steam" }, vm.OtherSchemes);
    }

    [Fact]
    public async Task SaveKeepsEnabledAppsThatAreNotInstalled()
    {
        var vm = await LoadedAsync();
        vm.Apps[0].Enabled = false;
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev2")));
        await vm.SaveCommand.ExecuteAsync(null);
        Assert.Equal(new[] { "zoom" }, _cli.LastSaved!.AppLinks);
        Assert.Equal(MessageKind.Success, vm.MessageKind);
    }

    [Fact]
    public async Task UseSchemeCreatesADraftLink()
    {
        var vm = await LoadedAsync();
        vm.SelectedScheme = "steam";
        vm.UseSchemeCommand.Execute(null);
        var link = Assert.Single(vm.CustomLinks);
        Assert.Equal("steam", link.Scheme);
        Assert.Equal("steam://{url}", link.Template);
        Assert.Same(link, vm.SelectedLink);
    }

    [Fact]
    public async Task SaveSendsCustomLinksNormalized()
    {
        var vm = await LoadedAsync();
        vm.AddLinkCommand.Execute(null);
        var link = vm.CustomLinks[0];
        link.Name = " Steam ";
        link.Scheme = "Steam";
        link.DomainsText = "store.steampowered.com\nsteamcommunity.com, ";
        link.Template = "steam://openurl/{url}";
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev2")));
        await vm.SaveCommand.ExecuteAsync(null);
        var saved = Assert.Single(_cli.LastSaved!.CustomDeepLinks!);
        Assert.Equal("Steam", saved.Name);
        Assert.Equal("steam", saved.Scheme);
        Assert.Equal(new[] { "store.steampowered.com", "steamcommunity.com" }, saved.Domains);
    }

    [Fact]
    public async Task AppsErrorIsShownAsError()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("rev1", LinkConfig.Empty)));
        _cli.AppsReply = FakeCli.Fail<AppsInfo>(CliExit.Error, "falló apps");
        var vm = new AppsViewModel(new ConfigSession(_cli, new FakeDialogs()), _cli);
        await vm.LoadCommand.ExecuteAsync(null);
        Assert.Equal("falló apps", vm.Message);
        Assert.Equal(MessageKind.Error, vm.MessageKind);
    }
}
