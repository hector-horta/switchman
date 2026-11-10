using Switchman.Core.Services;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Switchman.App.Services;

/// <summary>
/// Implementa los diálogos de Core con ContentDialog. Core llama desde cualquier hilo
/// (usa ConfigureAwait(false)); el diálogo se crea siempre en el hilo de la UI.
/// </summary>
internal sealed class ContentDialogService : IDialogService
{
    private XamlRoot? _root;
    private DispatcherQueue? _dispatcher;

    /// <summary>La fija MainWindow al cargarse, desde el hilo de la UI.</summary>
    public void Attach(XamlRoot root)
    {
        _root = root;
        _dispatcher = DispatcherQueue.GetForCurrentThread();
    }

    public Task<ConflictChoice> AskConflictAsync(string detail)
    {
        if (_root is null || _dispatcher is null)
        {
            return Task.FromResult(ConflictChoice.Cancel);
        }
        var answer = new TaskCompletionSource<ConflictChoice>(TaskCreationOptions.RunContinuationsAsynchronously);
        if (!_dispatcher.TryEnqueue(() => _ = ShowConflictAsync(detail, answer)))
        {
            answer.SetResult(ConflictChoice.Cancel);
        }
        return answer.Task;
    }

    /// <summary>Muestra el diálogo (hilo de la UI) y entrega la elección.</summary>
    private async Task ShowConflictAsync(string detail, TaskCompletionSource<ConflictChoice> answer)
    {
        try
        {
            var dialog = new ContentDialog
            {
                XamlRoot = _root,
                Title = "config.json cambió fuera de la app",
                Content = detail + Environment.NewLine + Environment.NewLine +
                    "Recargar descarta tus cambios. Sobrescribir guarda tu versión encima " +
                    "(si el archivo estaba dañado, se guarda una copia .bak).",
                PrimaryButtonText = "Recargar",
                SecondaryButtonText = "Sobrescribir",
                CloseButtonText = "Cancelar",
                DefaultButton = ContentDialogButton.Primary,
            };
            var result = await dialog.ShowAsync();
            answer.SetResult(result switch
            {
                ContentDialogResult.Primary => ConflictChoice.Reload,
                ContentDialogResult.Secondary => ConflictChoice.Overwrite,
                _ => ConflictChoice.Cancel,
            });
        }
        catch (Exception e) when (e is InvalidOperationException or System.Runtime.InteropServices.COMException)
        {
            AppServices.Logger.LogConflictDialogFailed(e);
            answer.SetResult(ConflictChoice.Cancel);
        }
    }
}
