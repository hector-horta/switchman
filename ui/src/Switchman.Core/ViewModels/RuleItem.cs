using CommunityToolkit.Mvvm.ComponentModel;
using Switchman.Core.Models;

namespace Switchman.Core.ViewModels;

/// <summary>Regla editable en la lista de la página Reglas.</summary>
public sealed partial class RuleItem : ObservableObject
{
    public required string Id { get; init; }

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Title))]
    public partial string Pattern { get; set; } = "";

    /// <summary>Nullable a propósito: el ComboBox escribe null cuando no hay selección.</summary>
    [ObservableProperty]
    public partial string? BrowserId { get; set; } = "";

    /// <summary>Nombre visible del navegador ("Helium", no "helium").</summary>
    [ObservableProperty]
    public partial string BrowserName { get; set; } = "";

    /// <summary>Texto de la lista: el dominio, o una indicación si todavía está vacío.</summary>
    public string Title => string.IsNullOrWhiteSpace(Pattern) ? "Nueva regla: escribe un dominio" : Pattern;

    /// <summary>Traduce id → nombre; la fija RulesViewModel, que conoce los navegadores detectados.</summary>
    public Func<string?, string>? NameLookup
    {
        get;
        set
        {
            field = value;
            RefreshName();
        }
    }

    partial void OnBrowserIdChanged(string? value) => RefreshName();

    private void RefreshName() => BrowserName = NameLookup?.Invoke(BrowserId) ?? BrowserId ?? "";

    [ObservableProperty]
    public partial bool StripTracking { get; set; } = true;

    public static RuleItem From(Rule rule) =>
        new() { Id = rule.Id, Pattern = rule.Pattern, BrowserId = rule.BrowserId, StripTracking = rule.StripTracking };

    public Rule ToRule() => new(Id, Pattern.Trim(), BrowserId ?? "", StripTracking);
}
