using Switchman.Core.Cli;
using Switchman.Core.Logging;
using Switchman.Core.Services;
using Microsoft.Extensions.Logging;

namespace Switchman.App.Services;

/// <summary>Raíz de composición: un CLI, una sesión de config y un log para toda la app.</summary>
internal static class AppServices
{
    private static readonly ILoggerFactory Logs = LoggerFactory.Create(builder => builder.AddProvider(
        new FileLoggerProvider(Path.Combine(DataDir(), "switchman-ui.log"))));

    /// <summary>
    /// Misma carpeta que usa el CLI (paths.rs): %LOCALAPPDATA%\Switchman si es una ruta absoluta;
    /// si no, la carpeta conocida de Windows.
    /// </summary>
    private static string DataDir()
    {
        var env = Environment.GetEnvironmentVariable("LOCALAPPDATA");
        var root = !string.IsNullOrWhiteSpace(env) && Path.IsPathFullyQualified(env)
            ? env
            : Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        return Path.Combine(root, "Switchman");
    }

    /// <summary>switchman-cli.exe va junto a Switchman.exe (lo copia el build y el instalador).</summary>
    public static string CliPath { get; } = Path.Combine(AppContext.BaseDirectory, "switchman-cli.exe");

    public static ContentDialogService Dialogs { get; } = new();

    public static ICliClient Cli { get; } = new CliClient(
        new LoggingRunner(new ProcessRunner(CliPath, TimeSpan.FromSeconds(10)), Logs.CreateLogger<LoggingRunner>()));

    public static ConfigSession Session { get; } = new(Cli, Dialogs);

    public static ILogger Logger { get; } = Logs.CreateLogger("Switchman.App");
}
