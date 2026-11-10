using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Switchman.Core.Cli;
using Switchman.Core.Models;
using Switchman.Core.Services;

namespace Switchman.Core.ViewModels;

/// <summary>Página Navegadores: navegador por defecto (config.json) y de respaldo (fallback.json).</summary>
public sealed partial class BrowsersViewModel(ConfigSession session, ICliClient cli) : ObservableObject
{
    private readonly ConfigSession _session = session;
    private readonly ICliClient _cli = cli;
    private string _savedFallbackId = "";

    public ObservableCollection<Browser> Browsers { get; } = [];

    /// <summary>Nullable a propósito: el ComboBox escribe null cuando no hay selección.</summary>
    [ObservableProperty]
    public partial string? DefaultBrowserId { get; set; } = "";

    /// <summary>Nullable a propósito: el ComboBox escribe null cuando no hay selección.</summary>
    [ObservableProperty]
    public partial string? FallbackBrowserId { get; set; } = "";

    [ObservableProperty]
    public partial string Message { get; set; } = "";

    /// <summary>Guarda el navegador por defecto y, si cambió, el de respaldo.</summary>
    [RelayCommand]
    private async Task SaveAsync(CancellationToken ct)
    {
        var config = (_session.Current?.Config ?? LinkConfig.Empty) with { DefaultBrowserId = DefaultBrowserId ?? "" };
        var outcome = await _session.SaveAsync(config, ct).ConfigureAwait(true);
        Message = outcome.Message;
        if (outcome.Status == SaveStatus.Reloaded)
        {
            DefaultBrowserId = _session.Current?.Config.DefaultBrowserId ?? "";
        }
        if (outcome.Status != SaveStatus.Saved || string.IsNullOrEmpty(FallbackBrowserId) || FallbackBrowserId == _savedFallbackId)
        {
            return;
        }
        var fallback = await _cli.SetFallbackAsync(FallbackBrowserId, ct).ConfigureAwait(true);
        if (fallback.IsOk)
        {
            _savedFallbackId = FallbackBrowserId;
        }
        else
        {
            Message = fallback.Error;
        }
    }

    /// <summary>Lee la config, los navegadores detectados y el respaldo actual.</summary>
    [RelayCommand]
    private async Task LoadAsync(CancellationToken ct)
    {
        // Primero la lista y después los ids: los ComboBox (SelectedValue TwoWay) escriben null en
        // el id cuando su lista se vacía, así que asignarlos antes de llenarla los borraba.
        var browsers = await _cli.DetectAsync(ct).ConfigureAwait(true);
        Browsers.Clear();
        if (browsers.IsOk) { foreach (var browser in browsers.Value!) { Browsers.Add(browser); } }
        else { Message = browsers.Error; }

        var config = await _session.LoadAsync(ct).ConfigureAwait(true);
        if (!config.IsOk) { Message = config.Error; }
        var defaultId = config.Value?.Config.DefaultBrowserId ?? "";
        if (defaultId.Length == 0 && Browsers.Count > 0)
        {
            // Config nueva: se propone el primero detectado; queda guardado al pulsar Guardar.
            defaultId = Browsers[0].Id;
        }
        DefaultBrowserId = defaultId;

        var status = await _cli.StatusAsync(ct).ConfigureAwait(true);
        _savedFallbackId = status.Value?.Fallback?.Id ?? "";
        FallbackBrowserId = _savedFallbackId;
    }
}
