using Switchman.App.Services;
using Microsoft.UI.Xaml;

namespace Switchman.App;

/// <summary>
/// Provides application-specific behavior to supplement the default Application class.
/// </summary>
public partial class App : Application
{
    private Window? _window;
    
    /// <summary>
    /// Initializes the singleton application object.  This is the first line of authored code
    /// executed, and as such is the logical equivalent of main() or WinMain().
    /// </summary>
    public App()
    {
        InitializeComponent();
        // Tres vías: WinUI no ve las excepciones que escapan por CoreMessaging (0xc000027b)
        // ni las de tareas sin observar; AppDomain registra lo que de todos modos va a cerrar la app.
        UnhandledException += (_, e) => AppServices.Logger.LogUnhandled(e.Exception);
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            if (e.ExceptionObject is Exception error)
            {
                AppServices.Logger.LogFatal(error);
            }
        };
        TaskScheduler.UnobservedTaskException += (_, e) =>
        {
            AppServices.Logger.LogUnobserved(e.Exception);
            e.SetObserved();
        };
    }

    /// <summary>
    /// Invoked when the application is launched.
    /// </summary>
    /// <param name="args">Details about the launch request and process.</param>
    protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        _window = new MainWindow();
        _window.Activate();
    }
}
