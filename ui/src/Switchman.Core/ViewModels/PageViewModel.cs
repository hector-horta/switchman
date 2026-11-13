using CommunityToolkit.Mvvm.ComponentModel;
using Switchman.Core.Services;

namespace Switchman.Core.ViewModels;

/// <summary>Tipo del mensaje de una página; la App lo traduce a <c>InfoBarSeverity</c>.</summary>
public enum MessageKind
{
    Info,
    Success,
    Warning,
    Error,
}

/// <summary>Base de los ViewModels de página: el mensaje del InfoBar y su severidad.</summary>
public abstract partial class PageViewModel : ObservableObject
{
    /// <summary>Resultado de la última acción, para el InfoBar ("" lo oculta).</summary>
    [ObservableProperty]
    public partial string Message { get; set; } = "";

    [ObservableProperty]
    public partial MessageKind MessageKind { get; set; }

    protected void Show(MessageKind kind, string message)
    {
        MessageKind = kind;
        Message = message;
    }

    protected void ShowOutcome(SaveOutcome outcome, string note = "")
    {
        Show(KindOf(outcome.Status), outcome.Status == SaveStatus.Saved ? outcome.Message + note : outcome.Message);
    }

    public static MessageKind KindOf(SaveStatus status)
    {
        return status switch
        {
            SaveStatus.Saved => MessageKind.Success,
            SaveStatus.Reloaded => MessageKind.Warning,
            SaveStatus.Cancelled => MessageKind.Info,
            _ => MessageKind.Error
        };
    }
}
