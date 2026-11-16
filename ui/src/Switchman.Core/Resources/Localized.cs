using System.Globalization;

namespace Switchman.Core.Resources;

/// <summary>Textos de <see cref="Strings"/> con argumentos, formateados con la cultura de la interfaz.</summary>
public static class Localized
{
    public static string Format(string format, params object?[] args) =>
        string.Format(CultureInfo.CurrentCulture, format, args);
}
