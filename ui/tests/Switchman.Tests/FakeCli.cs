using Switchman.Core.Cli;
using Switchman.Core.Models;
using Switchman.Core.Services;

namespace Switchman.Tests;

/// <summary>CLI falso: respuestas programadas y registro de llamadas.</summary>
internal sealed class FakeCli : ICliClient
{
    public static CliResponse<T> Ok<T>(T value) => new(CliExit.Ok, value, "");

    public static CliResponse<T> Fail<T>(CliExit exit, string error) => new(exit, default, error);

    public Queue<CliResponse<VersionedConfig>> GetConfigReplies { get; } = new();
    public Queue<CliResponse<SaveResult>> SetConfigReplies { get; } = new();

    public CliResponse<SaveResult> ResetReply { get; set; } = Ok(new SaveResult("none"));

    public CliResponse<IReadOnlyList<Browser>> DetectReply { get; set; } = Ok<IReadOnlyList<Browser>>([]);

    public CliResponse<RouteResult> TestUrlReply { get; set; } = Fail<RouteResult>(CliExit.Error, "sin respuesta");

    public CliResponse<AppStatus> StatusReply { get; set; } = Fail<AppStatus>(CliExit.Error, "sin respuesta");

    public CliResponse<string> CommandReply { get; set; } = Ok("");

    public List<string> Calls { get; } = [];

    public LinkConfig? LastSaved { get; private set; }

    public Task<CliResponse<VersionedConfig>> GetConfigAsync(CancellationToken ct)
    {
        Calls.Add("get-config");
        return Task.FromResult(GetConfigReplies.Dequeue());
    }

    public Task<CliResponse<SaveResult>> SetConfigAsync(LinkConfig config, string revision, CancellationToken ct)
    {
        Calls.Add($"set-config:{revision}");
        LastSaved = config;
        return Task.FromResult(SetConfigReplies.Dequeue());
    }

    public Task<CliResponse<SaveResult>> ResetConfigAsync(CancellationToken ct)
    {
        Calls.Add("reset-config");
        return Task.FromResult(ResetReply);
    }

    public Task<CliResponse<IReadOnlyList<Browser>>> DetectAsync(CancellationToken ct)
    {
        Calls.Add("detect");
        return Task.FromResult(DetectReply);
    }

    public Task<CliResponse<RouteResult>> TestUrlAsync(string url, CancellationToken ct)
    {
        Calls.Add($"test-url:{url}");
        return Task.FromResult(TestUrlReply);
    }

    public Task<CliResponse<AppStatus>> StatusAsync(CancellationToken ct)
    {
        Calls.Add("status");
        return Task.FromResult(StatusReply);
    }

    public Task<CliResponse<string>> SetFallbackAsync(string browserId, CancellationToken ct)
    {
        Calls.Add($"set-fallback:{browserId}");
        return Task.FromResult(CommandReply);
    }

    public Task<CliResponse<string>> RegisterAsync(CancellationToken ct)
    {
        Calls.Add("register");
        return Task.FromResult(CommandReply);
    }

    public Task<CliResponse<string>> UnregisterAsync(CancellationToken ct)
    {
        Calls.Add("unregister");
        return Task.FromResult(CommandReply);
    }
}

/// <summary>Diálogo falso: responde siempre <see cref="Answer"/>.</summary>
internal sealed class FakeDialogs : IDialogService
{
    public ConflictChoice Answer { get; set; } = ConflictChoice.Cancel;
    public int Asked { get; private set; }

    public Task<ConflictChoice> AskConflictAsync(string detail)
    {
        Asked++;
        return Task.FromResult(Answer);
    }
}
