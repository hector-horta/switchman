using CommunityToolkit.Mvvm.ComponentModel;
using Switchman.Core.Models;

namespace Switchman.Core.ViewModels;

/// <summary>Redirección editable en la lista de la página Redirecciones.</summary>
public sealed partial class RedirectItem : ObservableObject
{
    public required string Id { get; init; }

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Title))]
    public partial string From { get; set; } = "";

    [ObservableProperty]
    public partial string To { get; set; } = "";

    /// <summary>Texto de la lista: el origen, o una indicación si todavía está vacío.</summary>
    public string Title => string.IsNullOrWhiteSpace(From) ? Strings.RedirectsNewTitle : From;

    public static RedirectItem Of(Redirect redirect) => new() { Id = redirect.Id, From = redirect.From, To = redirect.To };

    public Redirect ToModel() => new(Id, From.Trim(), To.Trim());
}
