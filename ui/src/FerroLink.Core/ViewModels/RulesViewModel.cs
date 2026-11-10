using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FerroLink.Core.Cli;
using FerroLink.Core.Models;
using FerroLink.Core.Services;

namespace FerroLink.Core.ViewModels;

/// <summary>Página Reglas: lista editable de dominio/ruta → navegador.</summary>
public sealed partial class RulesViewModel(ConfigSession session, ICliClient cli) : ObservableObject
{
    private readonly ConfigSession _session = session;
    private readonly ICliClient _cli = cli;

    public ObservableCollection<RuleItem> Rules { get; } = [];

    public ObservableCollection<Browser> Browsers { get; } = [];

    /// <summary>Regla que se edita en el formulario de la página.</summary>
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(HasSelection))]
    public partial RuleItem? SelectedRule { get; set; }

    public bool HasSelection => SelectedRule is not null;

    /// <summary>Resultado de la última acción, para el InfoBar.</summary>
    [ObservableProperty]
    public partial string Message { get; set; } = "";

    /// <summary>Copia las reglas de la sesión a la lista editable.</summary>
    private void FillRules()
    {
        Rules.Clear();
        foreach (var rule in (_session.Current?.Config ?? LinkConfig.Empty).Rules)
        {
            Rules.Add(RuleItem.From(rule));
        }
    }

    /// <summary>Quita una regla de la lista (se aplica al guardar).</summary>
    [RelayCommand]
    private void RemoveRule(RuleItem rule)
    {
        Rules.Remove(rule);
        if (SelectedRule == rule)
        {
            SelectedRule = null;
        }
    }

    /// <summary>Guarda las reglas; si hubo conflicto y se recargó, refresca la lista.</summary>
    [RelayCommand]
    private async Task SaveAsync(CancellationToken ct)
    {
        var config = (_session.Current?.Config ?? LinkConfig.Empty) with { Rules = Rules.Select(r => r.ToRule()).ToList() };
        var note = "";
        if (config.DefaultBrowserId.Length == 0 && Browsers.Count > 0)
        {
            // Config nueva: sin navegador por defecto el CLI la rechaza. Se usa el primero detectado y se avisa.
            config = config with { DefaultBrowserId = Browsers[0].Id };
            note = $" · navegador por defecto: {Browsers[0].Name} (cámbialo en Navegadores)";
        }
        var outcome = await _session.SaveAsync(config, ct).ConfigureAwait(true);
        Message = outcome.Status == SaveStatus.Saved ? outcome.Message + note : outcome.Message;
        if (outcome.Status == SaveStatus.Reloaded)
        {
            FillRules();
        }
    }

    /// <summary>Agrega una regla vacía apuntando al navegador por defecto.</summary>
    [RelayCommand]
    private void AddRule()
    {
        var browserId = _session.Current?.Config.DefaultBrowserId ?? "";
        if (browserId.Length == 0 && Browsers.Count > 0)
        {
            browserId = Browsers[0].Id;
        }
        var rule = new RuleItem { Id = NextId(), Pattern = "", BrowserId = browserId };
        Rules.Add(rule);
        SelectedRule = rule;
    }

    /// <summary>Siguiente id libre de la forma "rN".</summary>
    private string NextId()
    {
        var max = 0;
        foreach (var rule in Rules)
        {
            if (rule.Id.StartsWith('r') && int.TryParse(rule.Id.AsSpan(1), System.Globalization.NumberStyles.None, System.Globalization.CultureInfo.InvariantCulture, out var n) && n > max)
            {
                max = n;
            }
        }
        return string.Create(System.Globalization.CultureInfo.InvariantCulture, $"r{max + 1}");
    }

    /// <summary>Carga las reglas de la sesión y los navegadores detectados.</summary>
    [RelayCommand]
    private async Task LoadAsync(CancellationToken ct)
    {
        var config = await _session.LoadAsync(ct).ConfigureAwait(true);
        if (!config.IsOk)
        {
            Message = config.Error;
        }
        FillRules();

        var browsers = await _cli.DetectAsync(ct).ConfigureAwait(true);
        Browsers.Clear();
        if (browsers.IsOk)
        {
            foreach (var browser in browsers.Value!)
            {
                Browsers.Add(browser);
            }
        }
        else
        {
            Message = browsers.Error;
        }
    }
}
