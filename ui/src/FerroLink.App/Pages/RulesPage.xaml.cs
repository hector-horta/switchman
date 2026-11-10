using FerroLink.App.Services;
using FerroLink.Core.ViewModels;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace FerroLink.App.Pages;

/// <summary>Vista de <see cref="RulesViewModel"/>; la lógica vive en el ViewModel.</summary>
public sealed partial class RulesPage : Page
{
    public RulesPage()
    {
        InitializeComponent();
    }

    public RulesViewModel ViewModel { get; } = new(AppServices.Session, AppServices.Cli);

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        ViewModel.LoadCommand.Execute(null);
    }
}
