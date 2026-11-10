using CommunityToolkit.Mvvm.ComponentModel;
using Switchman.Core.Models;

namespace Switchman.Core.ViewModels;

/// <summary>Regla editable en la lista de la página Reglas.</summary>
public sealed partial class RuleItem : ObservableObject
{
    public required string Id { get; init; }

    [ObservableProperty]
    public partial string Pattern { get; set; } = "";

    [ObservableProperty]
    public partial string BrowserId { get; set; } = "";

    [ObservableProperty]
    public partial bool StripTracking { get; set; } = true;

    public static RuleItem From(Rule rule) =>
        new() { Id = rule.Id, Pattern = rule.Pattern, BrowserId = rule.BrowserId, StripTracking = rule.StripTracking };

    public Rule ToRule() => new(Id, Pattern.Trim(), BrowserId, StripTracking);
}
