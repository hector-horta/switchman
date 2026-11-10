using Switchman.App.Services;
using Switchman.Core.ViewModels;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace Switchman.App.Pages;

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
