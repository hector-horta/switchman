using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FerroLink.Core.Cli;
using FerroLink.Core.Models;

namespace FerroLink.Core.ViewModels;

/// <summary>Página Probar enlace: qué navegador y qué URL limpia resultarían, sin abrir nada.</summary>
public sealed partial class TestLinkViewModel(ICliClient cli) : ObservableObject
{
    private readonly ICliClient _cli = cli;

    [ObservableProperty]
    [NotifyCanExecuteChangedFor(nameof(TestCommand))]
    public partial string Url { get; set; } = "";

    [ObservableProperty]
    public partial RouteResult? Result { get; set; }

    [ObservableProperty]
    public partial string Message { get; set; } = "";

    private bool CanTest() => !string.IsNullOrWhiteSpace(Url);

    /// <summary>Pregunta al CLI qué haría con el enlace.</summary>
    [RelayCommand(CanExecute = nameof(CanTest))]
    private async Task TestAsync(CancellationToken ct)
    {
        var response = await _cli.TestUrlAsync(Url.Trim(), ct).ConfigureAwait(true);
        Result = response.IsOk ? response.Value : null;
        Message = response.IsOk ? "" : response.Error;
    }
}
