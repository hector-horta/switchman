using System.Text.Json.Serialization;

namespace Switchman.Core.Models;

/// <summary>Navegador conocido por switchman (espejo de <c>config::Browser</c> en Rust).</summary>
public sealed record Browser(string Id, string Name, string ExePath, IReadOnlyList<string> Args);

/// <summary>Regla dominio/ruta → navegador.</summary>
public sealed record Rule(string Id, string Pattern, string BrowserId, bool StripTracking = true);

/// <summary>Contenido de <c>config.json</c>.</summary>
public sealed record LinkConfig(
    string DefaultBrowserId,
    bool StripTracking,
    IReadOnlyList<Rule> Rules,
    IReadOnlyList<Browser> CustomBrowsers,
    IReadOnlyList<string> ExtraTrackingParams,
    IReadOnlyList<string>? AppLinks = null,
    // Nombre fijo: SnakeCaseLower lo escribiría "custom_deep_links" y serde (Rust) lo ignoraría.
    [property: JsonPropertyName("custom_deeplinks")] IReadOnlyList<CustomDeepLink>? CustomDeepLinks = null)
{
    /// <summary>Igual que un config.json vacío (limpieza de rastreo activada).</summary>
    public static LinkConfig Empty { get; } = new("", true, [], [], []);
}

/// <summary>Deep link propio: enlaces de <see cref="Domains"/> → <see cref="Template"/> ({url} {host} {path} {query}).</summary>
public sealed record CustomDeepLink(string Id, string Name, string Scheme, IReadOnlyList<string> Domains, string Template);

/// <summary>Config junto con la revisión del archivo del que se leyó (<c>get-config</c>).</summary>
public sealed record VersionedConfig(string Revision, LinkConfig Config);
