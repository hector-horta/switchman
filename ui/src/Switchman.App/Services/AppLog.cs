using Microsoft.Extensions.Logging;

namespace Switchman.App.Services;

/// <summary>Mensajes de log de la App (generados en compilación).</summary>
internal static partial class AppLog
{
    [LoggerMessage(Level = LogLevel.Warning, Message = "No se pudo mostrar el diálogo de conflicto")]
    public static partial void LogConflictDialogFailed(this ILogger logger, Exception error);

    [LoggerMessage(Level = LogLevel.Error, Message = "Excepción no controlada en la UI")]
    public static partial void LogUnhandled(this ILogger logger, Exception error);

    [LoggerMessage(Level = LogLevel.Critical, Message = "Excepción fatal del proceso (la app se cierra)")]
    public static partial void LogFatal(this ILogger logger, Exception error);

    [LoggerMessage(Level = LogLevel.Error, Message = "Tarea con excepción no observada")]
    public static partial void LogUnobserved(this ILogger logger, Exception error);
}
