using System.Diagnostics;

namespace Switchman.Core.Cli;

/// <summary>Lanza <c>switchman-cli.exe</c> sin shell, con stdin/stdout/stderr redirigidos y timeout.</summary>
public sealed class ProcessRunner(string exePath, TimeSpan timeout) : IProcessRunner
{
    private readonly string _exePath = exePath;
    private readonly TimeSpan _timeout = timeout;

    /// <summary>Config del proceso: sin shell ni ventana, argumentos uno a uno.</summary>
    private ProcessStartInfo StartInfo(IReadOnlyList<string> args)
    {
        var psi = new ProcessStartInfo(_exePath)
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            // El CLI de Rust habla UTF-8; sin esto .NET usa la página de códigos de la consola
            // y rompe tildes en mensajes y en el JSON que se envía por stdin.
            StandardInputEncoding = new System.Text.UTF8Encoding(encoderShouldEmitUTF8Identifier: false),
            StandardOutputEncoding = System.Text.Encoding.UTF8,
            StandardErrorEncoding = System.Text.Encoding.UTF8,
        };
        foreach (var arg in args)
        {
            psi.ArgumentList.Add(arg);
        }
        return psi;
    }

    /// <summary>Escribe stdin y lo cierra; si el CLI ya terminó (tubería rota), lo ignora.</summary>
    private static async Task WriteStdinAsync(Process process, string? stdin, CancellationToken ct)
    {
        try
        {
            if (stdin is not null)
            {
                await process.StandardInput.WriteAsync(stdin.AsMemory(), ct).ConfigureAwait(false);
            }
            process.StandardInput.Close();
        }
        catch (IOException)
        {
            // El código de salida y stderr del CLI explican el fallo.
        }
    }

    /// <summary>Código de salida del proceso → <see cref="CliExit"/>; desconocidos cuentan como Error.</summary>
    private static CliExit ToExit(int code) =>
        code >= 0 && Enum.IsDefined(typeof(CliExit), code) ? (CliExit)code : CliExit.Error;

    /// <summary>
    /// Ejecuta el proceso con los argumentos proporcionados, redirigiendo streams y aplicando un timeout.
    /// </summary>
    public async Task<CliResult> RunAsync(IReadOnlyList<string> args, string? stdin, CancellationToken ct)
    {
        using var timeoutCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        timeoutCts.CancelAfter(_timeout);

        using var process = new Process { StartInfo = StartInfo(args) };

        try
        {
            process.Start();
            var stdoutTask = process.StandardOutput.ReadToEndAsync(timeoutCts.Token);
            var stderrTask = process.StandardError.ReadToEndAsync(timeoutCts.Token);

            try
            {
                await WriteStdinAsync(process, stdin, timeoutCts.Token).ConfigureAwait(false);
                await process.WaitForExitAsync(timeoutCts.Token).ConfigureAwait(false);
                var stdout = await stdoutTask.ConfigureAwait(false);
                var stderr = await stderrTask.ConfigureAwait(false);
                return new CliResult(ToExit(process.ExitCode), stdout.Trim(), stderr.Trim());
            }
            catch (OperationCanceledException) when (!ct.IsCancellationRequested)
            {
                process.Kill(entireProcessTree: true);
                return new CliResult(CliExit.Timeout, "", "switchman no respondió a tiempo");
            }
        }
        catch (System.ComponentModel.Win32Exception e)
        {
            return new CliResult(CliExit.NotStarted, "", e.Message);
        }
    }
}
