using FerroLink.Core.Cli;
using FerroLink.Core.Models;
using FerroLink.Core.Services;
using Xunit;

namespace FerroLink.Tests;

public sealed class ConfigSessionTests
{
    private static readonly LinkConfig Edited = new("edge", true, [new Rule("r1", "github.com", "chrome")], [], []);
    private static readonly LinkConfig OnDisk = new("brave", true, [], [], []);
    private readonly FakeCli _cli = new();
    private readonly FakeDialogs _dialogs = new();

    private ConfigSession Session() => new(_cli, _dialogs);

    [Fact]
    public async Task LoadSetsCurrentAndRaisesChanged()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("r1", OnDisk)));
        var session = Session();
        var changed = 0;
        session.Changed += (_, _) => changed++;
        await session.LoadAsync(CancellationToken.None);
        Assert.Equal("r1", session.Current!.Revision);
        Assert.Equal(1, changed);
    }

    [Fact]
    public async Task SaveSendsLoadedRevision()
    {
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("r1", OnDisk)));
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("r2")));
        var session = Session();
        await session.LoadAsync(CancellationToken.None);
        var outcome = await session.SaveAsync(Edited, CancellationToken.None);
        Assert.Equal(SaveStatus.Saved, outcome.Status);
        Assert.Equal("r2", session.Current!.Revision);
        Assert.Equal(Edited, session.Current.Config);
        Assert.Contains("set-config:r1", _cli.Calls);
    }

    [Fact]
    public async Task ConflictReloadDiscardsEdits()
    {
        _cli.SetConfigReplies.Enqueue(FakeCli.Fail<SaveResult>(CliExit.Conflict, "cambió"));
        _dialogs.Answer = ConflictChoice.Reload;
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("r3", OnDisk)));
        var session = Session();
        var outcome = await session.SaveAsync(Edited, CancellationToken.None);
        Assert.Equal(SaveStatus.Reloaded, outcome.Status);
        Assert.Equal(OnDisk, session.Current!.Config);
        Assert.Equal(1, _dialogs.Asked);
    }

    [Fact]
    public async Task ConflictOverwriteUsesFreshRevision()
    {
        _cli.SetConfigReplies.Enqueue(FakeCli.Fail<SaveResult>(CliExit.Conflict, "cambió"));
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("r4")));
        _dialogs.Answer = ConflictChoice.Overwrite;
        _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("r3", OnDisk)));
        var outcome = await Session().SaveAsync(Edited, CancellationToken.None);
        Assert.Equal(SaveStatus.Saved, outcome.Status);
        Assert.Equal(new[] { "set-config:none", "get-config", "set-config:r3" }, _cli.Calls);
    }

    [Fact]
    public async Task OverwriteCorruptFileResetsFirst()
    {
        _cli.SetConfigReplies.Enqueue(FakeCli.Fail<SaveResult>(CliExit.Conflict, "cambió"));
        _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("r5")));
        _dialogs.Answer = ConflictChoice.Overwrite;
        _cli.GetConfigReplies.Enqueue(FakeCli.Fail<VersionedConfig>(CliExit.Corrupt, "corrupto"));
        var outcome = await Session().SaveAsync(Edited, CancellationToken.None);
        Assert.Equal(SaveStatus.Saved, outcome.Status);
        Assert.Equal(new[] { "set-config:none", "get-config", "reset-config", "set-config:none" }, _cli.Calls);
    }

    [Fact]
    public async Task ConflictCancelKeepsState()
    {
        _cli.SetConfigReplies.Enqueue(FakeCli.Fail<SaveResult>(CliExit.Conflict, "cambió"));
        var session = Session();
        var outcome = await session.SaveAsync(Edited, CancellationToken.None);
        Assert.Equal(SaveStatus.Cancelled, outcome.Status);
        Assert.Null(session.Current);
    }

    [Fact]
    public async Task InvalidConfigFailsWithoutDialog()
    {
        _cli.SetConfigReplies.Enqueue(FakeCli.Fail<SaveResult>(CliExit.Error, "configuración inválida"));
        var outcome = await Session().SaveAsync(Edited, CancellationToken.None);
        Assert.Equal(SaveStatus.Failed, outcome.Status);
        Assert.Equal("configuración inválida", outcome.Message);
        Assert.Equal(0, _dialogs.Asked);
    }
}
