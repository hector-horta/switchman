using FerroLink.Core.Cli;

namespace FerroLink.Tests;

/// <summary>Runner falso: devuelve una respuesta fija y guarda la última llamada.</summary>
internal sealed class FakeRunner(CliResult reply) : IProcessRunner
{
    public IReadOnlyList<string> LastArgs { get; private set; } = [];
    public string? LastStdin { get; private set; }

    public Task<CliResult> RunAsync(IReadOnlyList<string> args, string? stdin, CancellationToken ct)
    {
        LastArgs = args;
        LastStdin = stdin;
        return Task.FromResult(reply);
    }
}
