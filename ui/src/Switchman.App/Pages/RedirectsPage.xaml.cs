using Switchman.App.Services;
using Switchman.Core.ViewModels;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace Switchman.App.Pages;

/// <summary>Vista de <see cref="RedirectsViewModel"/>; la lógica vive en el ViewModel.</summary>
public sealed partial class RedirectsPage : Page
{
    public RedirectsPage()
    {
        InitializeComponent();
    }

    public RedirectsViewModel ViewModel { get; } = new(AppServices.Session);

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        ViewModel.LoadCommand.Execute(null);
    }
}
