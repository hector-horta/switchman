using System.Runtime.InteropServices;

namespace Switchman.App;

/// <summary>Llamadas Win32 que WinUI no expone.</summary>
internal static partial class NativeMethods
{
    /// <summary>DPI de la ventana (96 = 100 %); funciona antes de mostrarla, a diferencia de XamlRoot.</summary>
    [LibraryImport("user32.dll")]
    internal static partial uint GetDpiForWindow(nint hwnd);
}
