namespace FerroLink.Core.Models;

/// <summary>Resultado de <c>test-url</c>.</summary>
public sealed record RouteResult(string BrowserId, string BrowserName, string? RuleId, bool Stripped, string Url);

/// <summary>Estado del registro de Windows (<c>status</c>).</summary>
public sealed record RegStatus(string? RegisteredExe, bool DefaultHttp, bool DefaultHttps);

/// <summary>Salida de <c>status</c>.</summary>
public sealed record AppStatus(RegStatus Registry, string SelfExe, Browser? Fallback);

/// <summary>Salida de <c>set-config</c>.</summary>
public sealed record SaveResult(string Revision);
