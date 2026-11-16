using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Switchman.Core.Models;
using Switchman.Core.ViewModels;

namespace Switchman.App;

/// <summary>Funciones para x:Bind (sin convertidores).</summary>
internal static class Ui
{
    public static bool HasText(string? text) => !string.IsNullOrWhiteSpace(text);

    /// <summary>Colapsa el InfoBar sin mensaje: cerrado igual ocupa el espaciado de su panel.</summary>
    public static Visibility VisibleIfText(string? text) => HasText(text) ? Visibility.Visible : Visibility.Collapsed;

    /// <summary>Severidad del InfoBar según el tipo de mensaje del ViewModel (Core no conoce WinUI).</summary>
    public static InfoBarSeverity SeverityOf(MessageKind kind) => kind switch
    {
        MessageKind.Success => InfoBarSeverity.Success,
        MessageKind.Warning => InfoBarSeverity.Warning,
        MessageKind.Error => InfoBarSeverity.Error,
        _ => InfoBarSeverity.Informational,
    };

    /// <summary>Visible si hay valor. Devuelve Visibility: la conversión implícita bool→Visibility
    /// sobre el resultado de una función genera código inválido en el compilador de XAML.</summary>
    public static Visibility VisibleIfSet(object? value) => value is null ? Visibility.Collapsed : Visibility.Visible;

    /// <summary>Recibe el resultado entero: x:Bind no llama a la función si el argumento es null.</summary>
    public static string RuleText(RouteResult? result) =>
        result?.RuleId is { Length: > 0 } id ? id
        : result?.AppName is { Length: > 0 } ? Strings.TestNoRuleApp
        : Strings.TestNoRuleDefault;

    public static string YesNo(bool value) => value ? Strings.Yes : Strings.No;

    /// <summary>Dónde se abre: la app del deep link si hay una, si no el navegador.</summary>
    public static string OpenedWith(RouteResult? result) =>
        result?.AppName is { Length: > 0 } app ? Localized.Format(Strings.TestAppSuffix, app) : result?.BrowserName ?? "";

    public static string DeepLinkText(RouteResult? result) =>
        result?.AppUri is { Length: > 0 } uri ? uri : Strings.TestNoDeepLink;

    public static string DefaultState(bool isDefault) => isDefault
        ? Strings.SettingsIsDefault
        : Strings.SettingsNotDefault;

    public static string RegisteredAt(AppStatus? status) => status?.Registry.RegisteredExe is { } exe
        ? Localized.Format(Strings.SettingsRegisteredAt, exe)
        : Strings.SettingsNotRegistered;
}
