namespace Switchman.Tests;

/// <summary>Límites de LOC de CLAUDE.md: páginas y ViewModels &lt;= 200, servicios &lt;= 300.</summary>
public sealed class LocLimitsTests
{
    private static DirectoryInfo SolutionRoot()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir is not null && !File.Exists(Path.Combine(dir.FullName, "Switchman.slnx")))
        {
            dir = dir.Parent;
        }
        return dir ?? throw new InvalidOperationException("no se encontró Switchman.slnx");
    }

    private static int LimitFor(string path)
    {
        return path.EndsWith(".xaml", StringComparison.OrdinalIgnoreCase) ||
               path.EndsWith(".xaml.cs", StringComparison.OrdinalIgnoreCase) ||
               path.Contains($"{Path.DirectorySeparatorChar}ViewModels{Path.DirectorySeparatorChar}", StringComparison.Ordinal)
            ? 200
            : 300;
    }

    private static bool IsGenerated(string path)
    {
        return path.Contains($"{Path.DirectorySeparatorChar}obj{Path.DirectorySeparatorChar}", StringComparison.Ordinal) ||
               path.Contains($"{Path.DirectorySeparatorChar}bin{Path.DirectorySeparatorChar}", StringComparison.Ordinal);
    }

    [Fact]
    public void SourceFilesRespectLocLimits()
    {
        var src = Path.Combine(SolutionRoot().FullName, "src");
        var files = Directory.EnumerateFiles(src, "*.*", SearchOption.AllDirectories)
            .Where(f => !IsGenerated(f) && (f.EndsWith(".cs", StringComparison.OrdinalIgnoreCase) || f.EndsWith(".xaml", StringComparison.OrdinalIgnoreCase)))
            .ToList();

        Assert.NotEmpty(files);

        var tooLong = files.Select(f => new { File = f, Lines = File.ReadLines(f).Count(), Limit = LimitFor(f) })
            .Where(x => x.Lines > x.Limit)
            .Select(x => $"{x.File} ({x.Lines} > {x.Limit})")
            .ToList();

        Assert.Empty(tooLong);
    }
}
