using CommunityToolkit.Mvvm.ComponentModel;

namespace Switchman.Core.ViewModels;

/// <summary>App del catálogo instalada, con su interruptor en la página Apps.</summary>
public sealed partial class AppToggle : ObservableObject
{
    public required string Id { get; init; }

    public required string Name { get; init; }

    /// <summary>Dominios que cubre, para la descripción ("wa.me, whatsapp.com").</summary>
    public required string Domains { get; init; }

    [ObservableProperty]
    public partial bool Enabled { get; set; }
}
