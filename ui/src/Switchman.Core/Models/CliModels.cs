namespace Switchman.Core.Models;

/// <summary>Resultado de <c>test-url</c>.</summary>
public sealed record RouteResult(
    string BrowserId,
    string BrowserName,
    string? RuleId,
    bool Stripped,
    string Url,
    string? AppName = null,
    string? AppUri = null);

/// <summary>Estado del registro de Windows (<c>status</c>).</summary>
public sealed record RegStatus(string? RegisteredExe, bool DefaultHttp, bool DefaultHttps);

/// <summary>Salida de <c>status</c>.</summary>
public sealed record AppStatus(RegStatus Registry, string SelfExe, Browser? Fallback);

/// <summary>Salida de <c>set-config</c>.</summary>
public sealed record SaveResult(string Revision);

/// <summary>App del catálogo de deep links y su estado en este equipo (<c>apps</c>).</summary>
public sealed record CatalogAppInfo(string Id, string Name, string Scheme, IReadOnlyList<string> Domains, bool Installed, bool Enabled);

/// <summary>Deep link propio y si su protocolo está instalado (<c>apps</c>).</summary>
public sealed record CustomDeepLinkInfo(string Id, string Name, string Scheme, IReadOnlyList<string> Domains, string Template, bool Installed);

/// <summary>Salida de <c>apps</c>: catálogo, deep links propios y otros protocolos instalados.</summary>
public sealed record AppsInfo(IReadOnlyList<CatalogAppInfo> Catalog, IReadOnlyList<CustomDeepLinkInfo> Custom, IReadOnlyList<string> OtherSchemes);

/// <summary>Problema de un campo de un deep link propio: <see cref="Field"/> name, scheme, domains o template.</summary>
public sealed record DeepLinkIssue(string Field, string Code);

/// <summary>Enlace de ejemplo probado: si es válido, si lo captura el deep link y la URI resultante.</summary>
public sealed record DeepLinkPreview(bool ValidUrl, bool Matches, string? Uri);

/// <summary>Salida de <c>check-deeplink</c>.</summary>
public sealed record DeepLinkCheck(IReadOnlyList<DeepLinkIssue> Issues, bool Installed, DeepLinkPreview? Preview);
