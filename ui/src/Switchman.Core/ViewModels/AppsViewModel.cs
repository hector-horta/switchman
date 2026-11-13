using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Switchman.Core.Cli;
using Switchman.Core.Models;
using Switchman.Core.Services;

namespace Switchman.Core.ViewModels;

/// <summary>
/// Página Apps: interruptor por app del catálogo de deep links (WhatsApp, Discord…), deep links
/// propios y los demás protocolos instalados, que se vuelven a buscar con "Buscar apps".
/// </summary>
public sealed partial class AppsViewModel(ConfigSession session, ICliClient cli) : PageViewModel
{
    private readonly ConfigSession _session = session;
    private readonly ICliClient _cli = cli;

    /// <summary>Apps activadas que ya no están instaladas: se conservan al guardar.</summary>
    private List<string> _hiddenEnabled = [];

    public ObservableCollection<AppToggle> Apps { get; } = [];

    public ObservableCollection<DeepLinkItem> CustomLinks { get; } = [];

    public ObservableCollection<string> OtherSchemes { get; } = [];

    /// <summary>Apps del catálogo que no están instaladas, para informarlo.</summary>
    [ObservableProperty]
    public partial string NotInstalled { get; set; } = "";

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(HasSelection))]
    public partial DeepLinkItem? SelectedLink { get; set; }

    public bool HasSelection => SelectedLink is not null;

    /// <summary>Protocolo elegido en "otros protocolos" para crear un deep link con él.</summary>
    [ObservableProperty]
    public partial string? SelectedScheme { get; set; }

    /// <summary>Agrega un deep link propio vacío y lo selecciona para editarlo.</summary>
    [RelayCommand]
    private void AddLink() => Select(new DeepLinkItem { Id = NextId() });

    /// <summary>Crea un deep link propio con el protocolo elegido en "otros protocolos".</summary>
    [RelayCommand]
    private void UseScheme()
    {
        if (string.IsNullOrWhiteSpace(SelectedScheme))
        {
            return;
        }
        Select(new DeepLinkItem { Id = NextId(), Name = SelectedScheme, Scheme = SelectedScheme, Template = SelectedScheme + "://{url}" });
    }

    /// <summary>Quita un deep link propio (se aplica al guardar).</summary>
    [RelayCommand]
    private void RemoveLink(DeepLinkItem link)
    {
        CustomLinks.Remove(link);
        if (SelectedLink == link)
        {
            SelectedLink = null;
        }
    }

    private void Select(DeepLinkItem link)
    {
        CustomLinks.Add(link);
        SelectedLink = link;
    }

    /// <summary>Primer id libre de la forma "dlN".</summary>
    private string NextId()
    {
        var n = 1;
        while (CustomLinks.Any(l => l.Id == string.Create(System.Globalization.CultureInfo.InvariantCulture, $"dl{n}")))
        {
            n++;
        }
        return string.Create(System.Globalization.CultureInfo.InvariantCulture, $"dl{n}");
    }

    /// <summary>Guarda las apps activadas y los deep links propios.</summary>
    [RelayCommand]
    private async Task SaveAsync(CancellationToken ct)
    {
        var appLinks = Apps.Where(a => a.Enabled).Select(a => a.Id).Concat(_hiddenEnabled).ToList();
        var links = CustomLinks.Select(l => l.ToModel()).ToList();
        var config = (_session.Current?.Config ?? LinkConfig.Empty) with { AppLinks = appLinks, CustomDeepLinks = links };
        var outcome = await _session.SaveAsync(config, ct).ConfigureAwait(true);
        ShowOutcome(outcome);
        if (outcome.Status == SaveStatus.Reloaded)
        {
            await LoadAsync(ct).ConfigureAwait(true);
        }
    }

    /// <summary>Lee la config y vuelve a buscar las apps y protocolos instalados ("Buscar apps").</summary>
    [RelayCommand]
    private async Task LoadAsync(CancellationToken ct)
    {
        var config = await _session.LoadAsync(ct).ConfigureAwait(true);
        if (!config.IsOk) { Show(MessageKind.Error, config.Error); }
        var info = await _cli.AppsAsync(ct).ConfigureAwait(true);
        if (!info.IsOk) { Show(MessageKind.Error, info.Error); return; }

        var enabled = _session.Current?.Config.AppLinks ?? [];
        Apps.Clear();
        foreach (var app in info.Value!.Catalog.Where(a => a.Installed))
        {
            Apps.Add(new AppToggle { Id = app.Id, Name = app.Name, Domains = string.Join(", ", app.Domains), Enabled = enabled.Contains(app.Id) });
        }

        _hiddenEnabled = enabled.Where(id => !Apps.Any(a => a.Id == id)).ToList();
        var missing = info.Value.Catalog.Where(a => !a.Installed).Select(a => a.Name).ToList();
        NotInstalled = missing.Count == 0 ? "" : "Del catálogo, sin instalar: " + string.Join(", ", missing);

        CustomLinks.Clear();
        foreach (var link in info.Value.Custom)
        {
            CustomLinks.Add(DeepLinkItem.From(link));
        }

        OtherSchemes.Clear();
        foreach (var scheme in info.Value.OtherSchemes)
        {
            OtherSchemes.Add(scheme);
        }
    }
}
