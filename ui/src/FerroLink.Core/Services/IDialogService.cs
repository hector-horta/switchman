namespace FerroLink.Core.Services;

/// <summary>Qué hacer cuando config.json cambió (o se dañó) desde que la UI lo leyó.</summary>
public enum ConflictChoice
{
    Reload,
    Overwrite,
    Cancel,
}

/// <summary>Diálogos que la App implementa con ContentDialog; Core no conoce WinUI.</summary>
public interface IDialogService
{
    Task<ConflictChoice> AskConflictAsync(string detail);
}
