using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using FerroLink.Core.Models;

namespace FerroLink.Core.Cli;

/// <summary>Implementación de <see cref="ICliClient"/> sobre <see cref="IProcessRunner"/>.</summary>
public sealed class CliClient(IProcessRunner runner) : ICliClient
{
    private static readonly CliJsonContext Json = CliJsonContext.Default;
    private readonly IProcessRunner _runner = runner;

    public Task<CliResponse<VersionedConfig>> GetConfigAsync(CancellationToken ct) =>
        CallAsync(["get-config"], null, Json.VersionedConfig, ct);

    public Task<CliResponse<SaveResult>> SetConfigAsync(LinkConfig config, string revision, CancellationToken ct) =>
        CallAsync(["set-config", "--revision", revision], JsonSerializer.Serialize(config, Json.LinkConfig), Json.SaveResult, ct);

    public Task<CliResponse<IReadOnlyList<Browser>>> DetectAsync(CancellationToken ct) =>
        CallAsync(["detect"], null, Json.IReadOnlyListBrowser, ct);

    public Task<CliResponse<RouteResult>> TestUrlAsync(string url, CancellationToken ct) =>
        CallAsync(["test-url", url], null, Json.RouteResult, ct);

    public Task<CliResponse<AppStatus>> StatusAsync(CancellationToken ct) =>
        CallAsync(["status"], null, Json.AppStatus, ct);

    public Task<CliResponse<string>> SetFallbackAsync(string browserId, CancellationToken ct) =>
        RunAsync(["set-fallback", browserId], ct);

    public Task<CliResponse<string>> RegisterAsync(CancellationToken ct) => RunAsync(["register"], ct);

    public Task<CliResponse<string>> UnregisterAsync(CancellationToken ct) => RunAsync(["unregister"], ct);

    /// <summary>
    /// Ejecuta una orden CLI a través del runner y procesa la respuesta según el tipo de datos esperado.
    /// </summary>
    private async Task<CliResponse<T>> CallAsync<T>(IReadOnlyList<string> args, string? stdin, JsonTypeInfo<T> type, CancellationToken ct)
    {
        var result = await _runner.RunAsync(args, stdin, ct).ConfigureAwait(false);
        if (result.Exit != CliExit.Ok)
            return new CliResponse<T>(result.Exit, default, ErrorText(result));

        try
        {
            var value = JsonSerializer.Deserialize(result.Stdout, type);
            return new CliResponse<T>(CliExit.Ok, value, value is null ? "respuesta vacía de ferro-link" : "");
        }
        catch (JsonException e)
        {
            return new CliResponse<T>(CliExit.Error, default, "respuesta inválida de ferro-link: " + e.Message);
        }
    }

    /// <summary>Comando sin salida JSON: devuelve stdout como texto.</summary>
    private async Task<CliResponse<string>> RunAsync(IReadOnlyList<string> args, CancellationToken ct)
    {
        var result = await _runner.RunAsync(args, null, ct).ConfigureAwait(false);
        return result.Exit == CliExit.Ok
            ? new CliResponse<string>(CliExit.Ok, result.Stdout, "")
            : new CliResponse<string>(result.Exit, null, ErrorText(result));
    }

    /// <summary>Mensaje para el usuario: stderr sin el prefijo "error: ", o el código si viene vacío.</summary>
    private static string ErrorText(CliResult result)
    {
        var text = result.Stderr.StartsWith("error: ", StringComparison.Ordinal) ? result.Stderr["error: ".Length..] : result.Stderr;
        return string.IsNullOrWhiteSpace(text) ? $"ferro-link terminó con {result.Exit}" : text;
    }
}
