using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Switchman.Core.Models;
using Switchman.Core.Services;

namespace Switchman.Core.ViewModels;

/// <summary>Página Redirecciones: enlace de origen (dominio y ruta) → enlace o app con que se abre.</summary>
public sealed partial class RedirectsViewModel(ConfigSession session) : PageViewModel
{
    private readonly ConfigSession _session = session;

    public ObservableCollection<RedirectItem> Redirects { get; } = [];

    /// <summary>Redirección que se edita en el formulario de la página.</summary>
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(HasSelection))]
    public partial RedirectItem? SelectedRedirect { get; set; }

    public bool HasSelection => SelectedRedirect is not null;

    /// <summary>Primer id libre de la forma "rdN".</summary>
    private string NextId()
    {
        var n = 1;
        while (Redirects.Any(r => r.Id == string.Create(System.Globalization.CultureInfo.InvariantCulture, $"rd{n}")))
        {
            n++;
        }
        return string.Create(System.Globalization.CultureInfo.InvariantCulture, $"rd{n}");
    }

    /// <summary>Copia las redirecciones de la sesión a la lista editable.</summary>
    private void FillRedirects()
    {
        Redirects.Clear();
        foreach (var redirect in (_session.Current?.Config ?? LinkConfig.Empty).Redirects ?? [])
        {
            Redirects.Add(RedirectItem.Of(redirect));
        }
        SelectedRedirect = null;
    }

    /// <summary>Agrega una redirección vacía y la selecciona para editarla.</summary>
    [RelayCommand]
    private void AddRedirect()
    {
        var redirect = new RedirectItem { Id = NextId() };
        Redirects.Add(redirect);
        SelectedRedirect = redirect;
    }

    /// <summary>Quita una redirección de la lista (se aplica al guardar).</summary>
    [RelayCommand]
    private void RemoveRedirect(RedirectItem redirect)
    {
        Redirects.Remove(redirect);
        if (SelectedRedirect == redirect)
        {
            SelectedRedirect = null;
        }
    }

    /// <summary>Guarda las redirecciones; si hubo conflicto y se recargó, refresca la lista.</summary>
    [RelayCommand]
    private async Task SaveAsync(CancellationToken ct)
    {
        var config = (_session.Current?.Config ?? LinkConfig.Empty) with { Redirects = Redirects.Select(r => r.ToModel()).ToList() };
        var outcome = await _session.SaveAsync(config, ct).ConfigureAwait(true);
        ShowOutcome(outcome);
        if (outcome.Status == SaveStatus.Reloaded)
        {
            FillRedirects();
        }
    }

    /// <summary>Carga las redirecciones de la sesión.</summary>
    [RelayCommand]
    private async Task LoadAsync(CancellationToken ct)
    {
        var config = await _session.LoadAsync(ct).ConfigureAwait(true);
        if (!config.IsOk)
        {
            Show(MessageKind.Error, config.Error);
        }
        FillRedirects();
    }
}
