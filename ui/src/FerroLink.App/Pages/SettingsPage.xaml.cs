using FerroLink.App.Services;
using FerroLink.Core.ViewModels;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace FerroLink.App.Pages;

/// <summary>Vista de <see cref="SettingsViewModel"/>; la lógica vive en el ViewModel.</summary>
public sealed partial class SettingsPage : Page
{
    public SettingsPage()
    {
        InitializeComponent();
    }

    public SettingsViewModel ViewModel { get; } = new(AppServices.Session, AppServices.Cli);

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        ViewModel.LoadCommand.Execute(null);
    }
}
