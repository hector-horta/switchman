using System.Text.Json.Serialization;
using Switchman.Core.Models;

namespace Switchman.Core.Cli;

/// <summary>
/// Serialización generada en compilación; snake_case como serde en el CLI. Los null se omiten:
/// serde usa el valor por defecto de un campo ausente, pero rechaza un null donde espera una lista.
/// </summary>
[JsonSourceGenerationOptions(
    PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower,
    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(VersionedConfig))]
[JsonSerializable(typeof(LinkConfig))]
[JsonSerializable(typeof(RouteResult))]
[JsonSerializable(typeof(AppStatus))]
[JsonSerializable(typeof(SaveResult))]
[JsonSerializable(typeof(AppsInfo))]
[JsonSerializable(typeof(IReadOnlyList<Browser>))]
internal sealed partial class CliJsonContext : JsonSerializerContext;
