using Switchman.Core.Models;

namespace Switchman.Core.Cli;

/// <summary>Contrato tipado del CLI de switchman; la UI nunca toca los JSON directamente.</summary>
public interface ICliClient
{
    Task<CliResponse<VersionedConfig>> GetConfigAsync(CancellationToken ct);
    Task<CliResponse<SaveResult>> SetConfigAsync(LinkConfig config, string revision, CancellationToken ct);
    Task<CliResponse<SaveResult>> ResetConfigAsync(CancellationToken ct);
    Task<CliResponse<IReadOnlyList<Browser>>> DetectAsync(CancellationToken ct);
    Task<CliResponse<RouteResult>> TestUrlAsync(string url, CancellationToken ct);
    Task<CliResponse<AppStatus>> StatusAsync(CancellationToken ct);
    Task<CliResponse<AppsInfo>> AppsAsync(CancellationToken ct);
    Task<CliResponse<string>> SetFallbackAsync(string browserId, CancellationToken ct);
    Task<CliResponse<string>> RegisterAsync(CancellationToken ct);
    Task<CliResponse<string>> UnregisterAsync(CancellationToken ct);
}
