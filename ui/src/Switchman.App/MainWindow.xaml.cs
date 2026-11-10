using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Switchman.App.Pages;
using Switchman.App.Services;

namespace Switchman.App;

public sealed partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        AppWindow.SetIcon("Assets/AppIcon.ico");
        FitToContent();
        if (AppWindow.Presenter is OverlappedPresenter presenter)
        {
            // Tamaño fijo: el contenido está diseñado para 1040x720 (sin bordes que estirar ni maximizar).
            presenter.IsResizable = false;
            presenter.IsMaximizable = false;
        }
    }

    /// <summary>
    /// Tamaño acorde al contenido (las páginas de formulario miden ~700 px), centrado en la
    /// pantalla. El tamaño por defecto de Windows dejaba media ventana vacía a la derecha.
    /// </summary>
    private void FitToContent()
    {
        const double Width = 1040, Height = 720;
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        var scale = NativeMethods.GetDpiForWindow(hwnd) / 96.0;
        var work = DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Primary).WorkArea;
        var w = Math.Min((int)(Width * scale), work.Width);
        var h = Math.Min((int)(Height * scale), work.Height);
        AppWindow.MoveAndResize(new Windows.Graphics.RectInt32(work.X + ((work.Width - w) / 2), work.Y + ((work.Height - h) / 2), w, h));
    }

    /// <summary>
    /// Los diálogos necesitan la raíz visual. La primera página la abre la selección inicial
    /// del menú (Reglas tiene IsSelected), no este evento: si no, se cargaría dos veces.
    /// </summary>
    private void Root_Loaded(object sender, RoutedEventArgs e)
    {
        AppServices.Dialogs.Attach(Root.XamlRoot);
        if (NavView.SettingsItem is NavigationViewItem settings)
        {
            settings.Content = "Ajustes";
        }
    }

    private void TitleBar_PaneToggleRequested(TitleBar sender, object args)
    {
        NavView.IsPaneOpen = !NavView.IsPaneOpen;
    }

    private void TitleBar_BackRequested(TitleBar sender, object args)
    {
        NavFrame.GoBack();
    }

    private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        if (args.IsSettingsSelected)
        {
            NavFrame.Navigate(typeof(SettingsPage));
        }
        else if (args.SelectedItem is NavigationViewItem item)
        {
            switch (item.Tag)
            {
                case "rules":
                    NavFrame.Navigate(typeof(RulesPage));
                    break;
                case "browsers":
                    NavFrame.Navigate(typeof(BrowsersPage));
                    break;
                case "test":
                    NavFrame.Navigate(typeof(TestLinkPage));
                    break;
                default:
                    throw new InvalidOperationException($"Unknown navigation item tag: {item.Tag}");
            }
        }
    }
}
