using System.Text.Json.Serialization;
using Switchman.Core.Models;

namespace Switchman.Core.Cli;

/// <summary>Serialización generada en compilación; snake_case como serde en el CLI.</summary>
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower)]
[JsonSerializable(typeof(VersionedConfig))]
[JsonSerializable(typeof(LinkConfig))]
[JsonSerializable(typeof(RouteResult))]
[JsonSerializable(typeof(AppStatus))]
[JsonSerializable(typeof(SaveResult))]
[JsonSerializable(typeof(IReadOnlyList<Browser>))]
internal sealed partial class CliJsonContext : JsonSerializerContext;
