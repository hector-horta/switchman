using System.Diagnostics;
using FerroLink.Core.Cli;
using Microsoft.Extensions.Logging;

namespace FerroLink.Core.Logging;

/// <summary>Decorador que registra cada llamada al CLI: comando, código de salida y duración.</summary>
public sealed partial class LoggingRunner(IProcessRunner inner, ILogger<LoggingRunner> logger) : IProcessRunner
{
    private readonly IProcessRunner _inner = inner;
    private readonly ILogger<LoggingRunner> _logger = logger;

    [LoggerMessage(Level = LogLevel.Information, Message = "ferro-link {Command} -> {Exit} en {ElapsedMs} ms")]
    private partial void LogCall(string command, CliExit exit, long elapsedMs);

    [LoggerMessage(Level = LogLevel.Warning, Message = "ferro-link {Command} falló ({Exit}): {Error}")]
    private partial void LogFailure(string command, CliExit exit, string error);

    public async Task<CliResult> RunAsync(IReadOnlyList<string> args, string? stdin, CancellationToken ct)
    {
        var command = args.Count > 0 ? args[0] : "";
        var watch = Stopwatch.StartNew();
        var result = await _inner.RunAsync(args, stdin, ct).ConfigureAwait(false);
        LogCall(command, result.Exit, watch.ElapsedMilliseconds);
        if (result.Exit != CliExit.Ok)
        {
            LogFailure(command, result.Exit, result.Stderr);
        }
        return result;
    }
}
