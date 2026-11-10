using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using FerroLink.App.Pages;
using FerroLink.App.Services;

namespace FerroLink.App;

public sealed partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        AppWindow.SetIcon("Assets/AppIcon.ico");
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
