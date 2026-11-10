namespace Switchman.Core.Cli;

/// <summary>Códigos de salida del CLI (ver <c>cli/src/main.rs</c>) más los fallos al lanzarlo.</summary>
public enum CliExit
{
    Ok = 0,
    Error = 1,
    Usage = 2,
    Corrupt = 3,
    Conflict = 4,
    Timeout = -1,
    NotStarted = -2,
}

/// <summary>Salida cruda de una ejecución del CLI.</summary>
public sealed record CliResult(CliExit Exit, string Stdout, string Stderr);

/// <summary>Respuesta tipada: <see cref="Value"/> solo si <see cref="Exit"/> es Ok.</summary>
public sealed record CliResponse<T>(CliExit Exit, T? Value, string Error)
{
    public bool IsOk => Exit == CliExit.Ok && Value is not null;
}
