using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Switchman.Core.Cli;
using Switchman.Core.Models;
using Switchman.Core.Services;

namespace Switchman.Core.ViewModels;

/// <summary>Página Ajustes: limpieza de rastreo, parámetros extra y registro en Windows.</summary>
public sealed partial class SettingsViewModel(ConfigSession session, ICliClient cli) : ObservableObject
{
    private readonly ConfigSession _session = session;
    private readonly ICliClient _cli = cli;
    private static readonly char[] ParamSeparators = ['\n', '\r', ','];

    [ObservableProperty]
    public partial bool StripTracking { get; set; } = true;

    /// <summary>Parámetros extra a quitar, uno por línea (o separados por comas).</summary>
    [ObservableProperty]
    public partial string ExtraParams { get; set; } = "";

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsDefault))]
    public partial AppStatus? Status { get; set; }

    [ObservableProperty]
    public partial string Message { get; set; } = "";

    /// <summary>Switchman es el navegador predeterminado para http y https.</summary>
    public bool IsDefault => Status?.Registry is { DefaultHttp: true, DefaultHttps: true };

    /// <summary>Registra Switchman como navegador y abre la Configuración de Windows.</summary>
    [RelayCommand]
    private Task RegisterAsync(CancellationToken ct) => ChangeRegistrationAsync(register: true, ct);

    /// <summary>Quita el registro de Switchman.</summary>
    [RelayCommand]
    private Task UnregisterAsync(CancellationToken ct) => ChangeRegistrationAsync(register: false, ct);

    /// <summary>Muestra en los campos la config de la sesión.</summary>
    private void ShowConfig()
    {
        var config = _session.Current?.Config ?? LinkConfig.Empty;
        StripTracking = config.StripTracking;
        ExtraParams = string.Join(Environment.NewLine, config.ExtraTrackingParams);
    }

    /// <summary>Relee el estado del registro (si falla, se muestra el error).</summary>
    private async Task RefreshStatusAsync(CancellationToken ct)
    {
        var status = await _cli.StatusAsync(ct).ConfigureAwait(true);
        Status = status.Value;
        if (!status.IsOk)
        {
            Message = status.Error;
        }
    }

    private async Task ChangeRegistrationAsync(bool register, CancellationToken ct)
    {
        var response = register ? await _cli.RegisterAsync(ct).ConfigureAwait(true) : await _cli.UnregisterAsync(ct).ConfigureAwait(true);
        if (!response.IsOk) { Message = response.Error; }
        else if (register) { Message = "Registrado: elige Switchman en la Configuración de Windows que se abrió"; }
        else { Message = "Switchman ya no está registrado como navegador"; }
        await RefreshStatusAsync(ct).ConfigureAwait(true);
    }

    /// <summary>Guarda la limpieza de rastreo y los parámetros extra.</summary>
    [RelayCommand]
    private async Task SaveAsync(CancellationToken ct)
    {
        var config = (_session.Current?.Config ?? LinkConfig.Empty) with { StripTracking = StripTracking, ExtraTrackingParams = ParseParams(ExtraParams) };
        var outcome = await _session.SaveAsync(config, ct).ConfigureAwait(true);
        Message = outcome.Message;
        if (outcome.Status is SaveStatus.Saved or SaveStatus.Reloaded)
        {
            ShowConfig();
        }
    }

    /// <summary>Lee la config y el estado del registro.</summary>
    [RelayCommand]
    private async Task LoadAsync(CancellationToken ct)
    {
        var response = await _session.LoadAsync(ct).ConfigureAwait(true);
        if (!response.IsOk) { Message = response.Error; }
        ShowConfig();
        await RefreshStatusAsync(ct).ConfigureAwait(true);
    }

    /// <summary>Texto libre → lista de parámetros sin vacíos ni duplicados.</summary>
    public static IReadOnlyList<string> ParseParams(string text)
    {
        return text.Split(ParamSeparators,StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Distinct(StringComparer.OrdinalIgnoreCase).ToList();
    }
}
