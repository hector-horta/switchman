using CommunityToolkit.Mvvm.ComponentModel;
using Switchman.Core.Models;

namespace Switchman.Core.ViewModels;

/// <summary>Deep link propio editable. Los dominios se escriben uno por línea (o separados por comas).</summary>
public sealed partial class DeepLinkItem : ObservableObject
{
    private static readonly char[] Separators = ['\n', '\r', ','];

    public required string Id { get; init; }

    /// <summary>El protocolo está registrado en este equipo (si no, el deep link no se usa).</summary>
    public bool Installed { get; init; } = true;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Title))]
    public partial string Name { get; set; } = "";

    [ObservableProperty]
    public partial string Scheme { get; set; } = "";

    [ObservableProperty]
    public partial string DomainsText { get; set; } = "";

    [ObservableProperty]
    public partial string Template { get; set; } = "";

    /// <summary>Texto de la lista: el nombre, o una indicación si todavía no tiene.</summary>
    public string Title => string.IsNullOrWhiteSpace(Name) ? "Nuevo deep link" : Name;

    public static DeepLinkItem From(CustomDeepLinkInfo info)
    {
        return new DeepLinkItem { Id = info.Id, Installed = info.Installed, Name = info.Name, Scheme = info.Scheme, DomainsText = string.Join(Environment.NewLine, info.Domains), Template = info.Template };
    }

    public CustomDeepLink ToModel()
    {
        return new CustomDeepLink(Id, Name.Trim(), Scheme.Trim().ToLowerInvariant(), DomainsText.Split(Separators, StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries), Template.Trim());
    }
}
