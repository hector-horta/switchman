using Microsoft.Extensions.Logging;

namespace Switchman.App.Services;

/// <summary>Mensajes de log de la App (generados en compilación).</summary>
internal static partial class AppLog
{
    [LoggerMessage(Level = LogLevel.Warning, Message = "Could not show the conflict dialog")]
    public static partial void LogConflictDialogFailed(this ILogger logger, Exception error);

    [LoggerMessage(Level = LogLevel.Error, Message = "Unhandled exception in the UI")]
    public static partial void LogUnhandled(this ILogger logger, Exception error);

    [LoggerMessage(Level = LogLevel.Critical, Message = "Fatal process exception (the app is closing)")]
    public static partial void LogFatal(this ILogger logger, Exception error);

    [LoggerMessage(Level = LogLevel.Error, Message = "Unobserved task exception")]
    public static partial void LogUnobserved(this ILogger logger, Exception error);
}
