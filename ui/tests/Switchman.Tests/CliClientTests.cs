using Switchman.Core.Cli;
using Switchman.Core.Models;

namespace Switchman.Tests;

public sealed class CliClientTests
{
    private static CliClient Client(FakeRunner runner) => new(runner);

    [Fact] public async Task GetConfigParsesSnakeCaseJson()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Ok, """{"revision":"abc","config":{"default_browser_id":"edge","strip_tracking":true,"rules":[{"id":"r1","pattern":"github.com","browser_id":"chrome","strip_tracking":false}],"custom_browsers":[],"extra_tracking_params":["ref"]}}""", ""));
        var r = await Client(runner).GetConfigAsync(CancellationToken.None);
        Assert.True(r.IsOk);
        Assert.Equal("abc", r.Value!.Revision);
        Assert.Equal("edge", r.Value.Config.DefaultBrowserId);
        Assert.False(r.Value.Config.Rules[0].StripTracking);
        Assert.Equal(new[] { "ref" }, r.Value.Config.ExtraTrackingParams);
        Assert.Equal(new[] { "get-config" }, runner.LastArgs);
    }

    [Fact] public async Task SetConfigSendsRevisionAndSnakeCaseStdin()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Ok, """{"revision":"new"}""", ""));
        var config = new LinkConfig("edge", true, [new Rule("r1", "github.com", "chrome")], [], []);
        var r = await Client(runner).SetConfigAsync(config, "old", CancellationToken.None);
        Assert.Equal("new", r.Value!.Revision);
        Assert.Equal(new[] { "set-config", "--revision", "old" }, runner.LastArgs);
        Assert.Contains("\"default_browser_id\":\"edge\"", runner.LastStdin, StringComparison.Ordinal);
    }

    [Fact] public async Task ConflictKeepsExitCodeAndStripsErrorPrefix()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Conflict, "", "error: config.json cambió"));
        var r = await Client(runner).SetConfigAsync(new LinkConfig("edge", true, [], [], []), "old", CancellationToken.None);
        Assert.False(r.IsOk);
        Assert.Equal(CliExit.Conflict, r.Exit);
        Assert.Equal("config.json cambió", r.Error);
    }

    [Fact] public async Task CorruptConfigIsReported()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Corrupt, "", "error: config.json corrupto: x"));
        var r = await Client(runner).GetConfigAsync(CancellationToken.None);
        Assert.Equal(CliExit.Corrupt, r.Exit);
        Assert.Null(r.Value);
    }

    [Fact] public async Task InvalidJsonIsAnError()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Ok, "no json", ""));
        var r = await Client(runner).StatusAsync(CancellationToken.None);
        Assert.Equal(CliExit.Error, r.Exit);
        Assert.StartsWith("respuesta inválida", r.Error, StringComparison.Ordinal);
    }

    [Fact] public async Task EmptyStderrUsesExitCode()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Timeout, "", ""));
        var r = await Client(runner).DetectAsync(CancellationToken.None);
        Assert.Contains("Timeout", r.Error, StringComparison.Ordinal);
    }

    [Fact] public async Task TestUrlPassesUrlAsSingleArgument()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Ok, """{"browser_id":"chrome","browser_name":"Google Chrome","rule_id":null,"stripped":true,"url":"https://a.com/?q=1"}""", ""));
        var url = "https://a.com/?q=1&utm_source=x y";
        var r = await Client(runner).TestUrlAsync(url, CancellationToken.None);
        Assert.Equal(new[] { "test-url", url }, runner.LastArgs);
        Assert.Null(r.Value!.RuleId);
        Assert.True(r.Value.Stripped);
    }

    [Fact]
    public async Task AppsParsesCatalogCustomAndOtherSchemes()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Ok, """{"catalog":[{"id":"whatsapp","name":"WhatsApp","scheme":"whatsapp","domains":["wa.me"],"installed":true,"enabled":false}],"custom":[{"id":"steam","name":"Steam","scheme":"steam","domains":["store.steampowered.com"],"template":"steam://openurl/{url}","installed":true}],"other_schemes":["bitwarden"]}""", ""));
        var r = await Client(runner).AppsAsync(CancellationToken.None);
        Assert.True(r.Value!.Catalog[0].Installed);
        Assert.Equal("steam://openurl/{url}", r.Value.Custom[0].Template);
        Assert.Equal(["bitwarden"], r.Value.OtherSchemes);
        Assert.Equal(["apps"], runner.LastArgs);
    }

    [Fact]
    public async Task ConfigWithoutDeepLinksDoesNotSendNulls()
    {
        var runner = new FakeRunner(new CliResult(CliExit.Ok, """{"revision":"r"}""", ""));
        await Client(runner).SetConfigAsync(new LinkConfig("edge", true, [], [], []), "none", CancellationToken.None);
        Assert.DoesNotContain("null", runner.LastStdin, StringComparison.Ordinal);
        await Client(runner).SetConfigAsync(new LinkConfig("edge", true, [], [], [], ["whatsapp"], []), "none", CancellationToken.None);
        Assert.Contains("\"app_links\":[\"whatsapp\"]", runner.LastStdin, StringComparison.Ordinal);
        // Los nombres deben ser los de serde en Rust (config_types.rs), no los de la convención de C#.
        await Client(runner).SetConfigAsync(new LinkConfig("edge", true, [], [], [], [], [new CustomDeepLink("dl1", "Steam", "steam", ["a.com"], "steam://{url}")]), "none", CancellationToken.None);
        Assert.Contains("\"custom_deeplinks\":[{\"id\":\"dl1\"", runner.LastStdin, StringComparison.Ordinal);
    }
}
