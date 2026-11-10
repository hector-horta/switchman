using FerroLink.Core.Cli;
using FerroLink.Core.Models;

namespace FerroLink.Core.Services;

/// <summary>Resultado de guardar desde la UI.</summary>
public enum SaveStatus
{
    Saved,
    Reloaded,
    Cancelled,
    Failed,
}

/// <summary>Estado de guardado y mensaje para el usuario.</summary>
public sealed record SaveOutcome(SaveStatus Status, string Message);

/// <summary>
/// Config compartida por las páginas: la última leída y su revisión. Guarda con control
/// optimista; ante conflicto o config corrupta pregunta al usuario (recargar o sobrescribir).
/// </summary>
public sealed class ConfigSession(ICliClient cli, IDialogService dialogs)
{
    private readonly ICliClient _cli = cli;
    private readonly IDialogService _dialogs = dialogs;

    /// <summary>Última config leída o guardada; null hasta el primer LoadAsync exitoso.</summary>
    public VersionedConfig? Current { get; private set; }

    /// <summary>Se dispara cuando <see cref="Current"/> cambia (para que las páginas se refresquen).</summary>
    public event EventHandler? Changed;

    /// <summary>
    /// Carga la configuración desde el archivo JSON de manera asíncrona.
    /// Si hay un error al leer, mantiene el estado actual sin modificar.
    /// </summary>
    public async Task<CliResponse<VersionedConfig>> LoadAsync(CancellationToken ct)
    {
        var response = await _cli.GetConfigAsync(ct).ConfigureAwait(false);
        if (response.IsOk)
        {
            Accept(response.Value!);
        }

        return response;
    }

    /// <summary>
    /// Guarda la configuración en el sistema con manejo de conflictos y mensajes al usuario.
    /// </summary>
    public async Task<SaveOutcome> SaveAsync(LinkConfig config, CancellationToken ct)
    {
        var revision = Current?.Revision ?? "none";
        var response = await _cli.SetConfigAsync(config, revision, ct).ConfigureAwait(false);
        if (response.IsOk)
        {
            Accept(new VersionedConfig(response.Value!.Revision, config));
            return new SaveOutcome(SaveStatus.Saved, "Guardado");
        }

        if (response.Exit != CliExit.Conflict)
        {
            return new SaveOutcome(SaveStatus.Failed, response.Error);
        }

        var choice = await _dialogs.AskConflictAsync(response.Error).ConfigureAwait(false);
        return choice switch
        {
            ConflictChoice.Reload => await ReloadAfterConflictAsync(ct).ConfigureAwait(false),
            ConflictChoice.Overwrite => await OverwriteAsync(config, ct).ConfigureAwait(false),
            _ => new SaveOutcome(SaveStatus.Cancelled, "No se guardó"),
        };
    }

    /// <summary>
    /// Sobrescribe la configuración existente con la nueva, recuperando la última versión válida del archivo.
    /// </summary>
    private async Task<SaveOutcome> OverwriteAsync(LinkConfig config, CancellationToken ct)
    {
        var current = await _cli.GetConfigAsync(ct).ConfigureAwait(false);
        string revision;
        if (current.IsOk)
        {
            revision = current.Value!.Revision;
        }
        else if (current.Exit == CliExit.Corrupt)
        {
            var reset = await _cli.ResetConfigAsync(ct).ConfigureAwait(false);
            if (!reset.IsOk)
            {
                return new SaveOutcome(SaveStatus.Failed, reset.Error);
            }

            revision = reset.Value!.Revision;
        }
        else
        {
            return new SaveOutcome(SaveStatus.Failed, current.Error);
        }

        var response = await _cli.SetConfigAsync(config, revision, ct).ConfigureAwait(false);
        if (!response.IsOk)
        {
            return new SaveOutcome(SaveStatus.Failed, response.Error);
        }

        Accept(new VersionedConfig(response.Value!.Revision, config));
        return new SaveOutcome(SaveStatus.Saved, "Guardado (se sobrescribió config.json)");
    }

    /// <summary>El usuario eligió recargar: se descarta lo editado y se lee config.json.</summary>
    private async Task<SaveOutcome> ReloadAfterConflictAsync(CancellationToken ct)
    {
        var response = await LoadAsync(ct).ConfigureAwait(false);
        return response.IsOk
            ? new SaveOutcome(SaveStatus.Reloaded, "Se recargó config.json; tus cambios no se guardaron")
            : new SaveOutcome(SaveStatus.Failed, response.Error);
    }

    /// <summary>Adopta la config y su revisión, y avisa a las páginas.</summary>
    private void Accept(VersionedConfig config)
    {
        Current = config;
        Changed?.Invoke(this, EventArgs.Empty);
    }
}
