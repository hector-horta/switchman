using FerroLink.App.Services;
using FerroLink.Core.ViewModels;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace FerroLink.App.Pages;

/// <summary>Vista de <see cref="BrowsersViewModel"/>; la lógica vive en el ViewModel.</summary>
public sealed partial class BrowsersPage : Page
{
    public BrowsersPage()
    {
        InitializeComponent();
    }

    public BrowsersViewModel ViewModel { get; } = new(AppServices.Session, AppServices.Cli);

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        ViewModel.LoadCommand.Execute(null);
    }
}
