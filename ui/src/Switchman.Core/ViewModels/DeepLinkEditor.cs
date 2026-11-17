using System.ComponentModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Switchman.Core.Cli;
using Switchman.Core.Models;

namespace Switchman.Core.ViewModels;

/// <summary>
/// Editor de un deep link propio: errores junto a cada campo, plantillas prearmadas y vista previa de
/// un enlace de ejemplo. Revisa con <c>check-deeplink</c>, así la regla es la misma que al hacer clic.
/// </summary>
public sealed partial class DeepLinkEditor : ObservableObject, IDisposable
{
    private readonly ICliClient _cli;
    private readonly TimeSpan _delay;
    private CancellationTokenSource? _pending;
    private string _lastScheme;

    public DeepLinkEditor(ICliClient cli, DeepLinkItem link, TimeSpan delay)
    {
        _cli = cli;
        _delay = delay;
        Link = link;
        _lastScheme = link.Scheme.Trim();
        ExampleUrl = link.ToModel().Domains is [var domain, ..] ? $"https://{domain}/" : "";
        link.PropertyChanged += OnLinkChanged;
    }

    public DeepLinkItem Link { get; }

    [ObservableProperty]
    public partial string ExampleUrl { get; set; }

    [ObservableProperty]
    public partial string NameError { get; set; } = "";

    [ObservableProperty]
    public partial string SchemeError { get; set; } = "";

    [ObservableProperty]
    public partial string DomainsError { get; set; } = "";

    [ObservableProperty]
    public partial string TemplateError { get; set; } = "";

    /// <summary>URI en la que se convierte el ejemplo; vacía si no se puede calcular.</summary>
    [ObservableProperty]
    [NotifyCanExecuteChangedFor(nameof(TryCommand))]
    public partial string PreviewUri { get; set; } = "";

    /// <summary>Aviso sobre el ejemplo (no es un enlace, no es de estos dominios, app sin instalar).</summary>
    [ObservableProperty]
    public partial string PreviewNote { get; set; } = "";

    partial void OnExampleUrlChanged(string value) => Schedule();

    private bool CanTry() => PreviewUri.Length > 0;

    /// <summary>Plantilla que pasa el enlace entero a la app (p. ej. steam://{url}; se puede ajustar).</summary>
    [RelayCommand]
    private void UseWholeLink() => Link.Template = Link.Scheme.Trim() + "://{url}";

    /// <summary>Plantilla que deja el enlace igual pero con el protocolo de la app en vez de https://.</summary>
    [RelayCommand]
    private void UseReplaceScheme() => Link.Template = Link.Scheme.Trim() + "://{host}/{path}?{query}";

    /// <summary>Abre la URI de la vista previa en la app (sin guardar nada).</summary>
    [RelayCommand(CanExecute = nameof(CanTry))]
    private async Task TryAsync(CancellationToken ct)
    {
        var response = await _cli.OpenUriAsync(PreviewUri, ct).ConfigureAwait(true);
        if (!response.IsOk)
        {
            PreviewNote = response.Error;
        }
    }

    /// <summary>Deja de seguir los cambios del deep link (al elegir otro en la lista).</summary>
    public void Dispose()
    {
        Link.PropertyChanged -= OnLinkChanged;
        _pending?.Cancel();
        _pending?.Dispose();
        _pending = null;
    }

    /// <summary>
    /// Revisa de nuevo tras cada cambio; si cambia el protocolo, también cambia el comienzo de la plantilla.
    /// </summary>
    private void OnLinkChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(DeepLinkItem.Scheme))
        {
            var scheme = Link.Scheme.Trim();
            var previous = _lastScheme;
            _lastScheme = scheme;
            if (previous.Length > 0 && Link.Template.StartsWith(previous + ":", StringComparison.OrdinalIgnoreCase))
            {
                Link.Template = scheme + Link.Template[previous.Length..];
            }
            else if (Link.Template.Trim().Length == 0 && scheme.Length > 0)
            {
                Link.Template = scheme + "://{url}";
            }
            if (Link.Name.Trim().Length == 0 || string.Equals(Link.Name, previous, StringComparison.Ordinal))
            {
                Link.Name = scheme;
            }
        }

        Schedule();
    }

    /// <summary>
    /// Programa una revisión; un cambio nuevo cancela la anterior.
    /// </summary>
    private void Schedule()
    {
        _pending?.Cancel();
        _pending?.Dispose();
        _pending = new CancellationTokenSource();
        _ = RunDebouncedAsync(_pending.Token);
    }

    /// <summary>
    /// Espera a que se dejen de hacer cambios y revisa el deep link.
    /// </summary>
    private async Task RunDebouncedAsync(CancellationToken ct)
    {
        try
        {
            await Task.Delay(_delay, ct).ConfigureAwait(true);
            await RefreshAsync(ct).ConfigureAwait(true);
        }
        catch (OperationCanceledException)
        {
            // Llegó otro cambio: lo revisa la llamada siguiente.
        }
    }

    /// <summary>
    /// Revisa el deep link con el CLI y actualiza los errores de cada campo y la vista previa.
    /// </summary>
    public async Task RefreshAsync(CancellationToken ct)
    {
        var model = Link.ToModel();
        var example = string.IsNullOrWhiteSpace(ExampleUrl) ? null : ExampleUrl.Trim();
        var response = await _cli.CheckDeepLinkAsync(model, example, ct).ConfigureAwait(true);

        if (!response.IsOk || response.Value is null)
        {
            PreviewUri = "";
            PreviewNote = response.Error;
            return;
        }

        var check = response.Value;

        NameError = MessageFor(check, "name", model.Scheme);
        SchemeError = MessageFor(check, "scheme", model.Scheme);
        DomainsError = MessageFor(check, "domains", model.Scheme);
        TemplateError = MessageFor(check, "template", model.Scheme);

        var preview = check.Preview;
        PreviewUri = preview is { ValidUrl: true, Uri: { } uri } && SchemeError.Length == 0 && TemplateError.Length == 0 ? uri : "";
        PreviewNote = preview is null ? "" : !preview.ValidUrl ? Strings.DlExampleInvalid : !preview.Matches ? Strings.DlExampleNoMatch : !check.Installed && SchemeError.Length == 0 ? Localized.Format(Strings.DlNotInstalled, model.Scheme) : "";
    }

    /// <summary>Texto del primer problema de <paramref name="field"/>, o vacío si no tiene.</summary>
    private static string MessageFor(DeepLinkCheck check, string field, string scheme)
    {
        return check.Issues.FirstOrDefault(i => string.Equals(i.Field, field, StringComparison.Ordinal)) switch
        {
            null => "",
            { Code: "name_invalid" } => Strings.DlNameInvalid,
            { Code: "scheme_invalid" } => Strings.DlSchemeInvalid,
            { Code: "template_scheme" } => Localized.Format(Strings.DlTemplateScheme, scheme),
            { Code: "template_too_long" } => Strings.DlTemplateTooLong,
            { Code: "domains_count" } => Strings.DlDomainsCount,
            { Code: "domain_path" } => Strings.DlDomainPath,
            _ => Strings.DlDomainInvalid
        };
    }
}
