namespace Switchman.Core.Cli;

/// <summary>Ejecuta el CLI; se reemplaza por un falso en los tests.</summary>
public interface IProcessRunner
{
    Task<CliResult> RunAsync(IReadOnlyList<string> args, string? stdin, CancellationToken ct);
}
