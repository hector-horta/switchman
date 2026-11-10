using Microsoft.Extensions.Logging;

namespace Switchman.Core.Logging;

/// <summary>
/// Log en archivo (una línea por evento) junto al del CLI: %LOCALAPPDATA%\Switchman\switchman-ui.log.
/// Al pasar de <see cref="MaxBytes"/> se renombra a <c>.1</c> y se empieza otro. Nunca lanza.
/// </summary>
public sealed class FileLoggerProvider(string path) : ILoggerProvider
{
    public const long MaxBytes = 1024 * 1024;

    private readonly string _path = path;
    private readonly Lock _gate = new();

    public ILogger CreateLogger(string categoryName) => new FileLogger(this, categoryName);

    /// <summary>Agrega una línea, rotando a <c>.1</c> si el archivo creció de más.</summary>
    internal void Write(string line)
    {
        lock (_gate)
        {
            try
            {
                var dir = Path.GetDirectoryName(_path);
                if (!string.IsNullOrEmpty(dir))
                {
                    Directory.CreateDirectory(dir);
                }
                var info = new FileInfo(_path);
                if (info.Exists && info.Length > MaxBytes)
                {
                    File.Move(_path, _path + ".1", overwrite: true);
                }
                File.AppendAllText(_path, line + Environment.NewLine);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException)
            {
                // El log nunca debe tumbar la UI.
            }
        }
    }

    public void Dispose()
    {
        // Sin recursos abiertos: cada escritura abre y cierra el archivo.
    }
}

/// <summary>Logger de una categoría; formatea y delega la escritura al proveedor.</summary>
internal sealed class FileLogger(FileLoggerProvider provider, string category) : ILogger
{
    private readonly FileLoggerProvider _provider = provider;
    private readonly string _category = category;

    public IDisposable? BeginScope<TState>(TState state) where TState : notnull => null;

    public bool IsEnabled(LogLevel logLevel) => logLevel >= LogLevel.Information;

    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
        if (!IsEnabled(logLevel)) { return; }
        var message = formatter(state, exception);
        var line = string.Create(System.Globalization.CultureInfo.InvariantCulture, $"{DateTimeOffset.Now:O} {logLevel} {_category}: {message}");
        if (exception is not null) { line += " | " + exception.GetType().Name + ": " + exception.Message; }
        _provider.Write(line);
    }
}
