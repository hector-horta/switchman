using Switchman.App.Services;
using Switchman.Core.ViewModels;
using Microsoft.UI.Xaml.Controls;

namespace Switchman.App.Pages;

/// <summary>Vista de <see cref="TestLinkViewModel"/>; la lógica vive en el ViewModel.</summary>
public sealed partial class TestLinkPage : Page
{
    public TestLinkPage()
    {
        InitializeComponent();
    }

    public TestLinkViewModel ViewModel { get; } = new(AppServices.Cli);
}
