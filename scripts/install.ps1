<#
.SYNOPSIS
  Installs (or uninstalls) Switchman for the current user, without administrator rights.
.DESCRIPTION
  From the release zip it uses the bundled executables; from the source tree it builds the Rust
  CLI and publishes the WinUI 3 app. It copies both to -Dest, creates the Start menu shortcut
  and registers Switchman as a browser (Windows Settings opens so you can choose it).
  -Uninstall removes the registration, the shortcut and the folder. It never deletes the user's
  configuration (%LOCALAPPDATA%\Switchman).
#>
param(
    [switch]$Uninstall,
    [string]$Dest = (Join-Path $env:LOCALAPPDATA 'Programs\Switchman')
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$cli = Join-Path $Dest 'switchman-cli.exe'
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'Switchman.lnk'

function Invoke-Cli([string]$Command) {
    $p = Start-Process -FilePath $cli -ArgumentList $Command -Wait -PassThru -WindowStyle Hidden
    if ($p.ExitCode -ne 0) { throw "switchman-cli $Command exited with $($p.ExitCode)" }
}

function Stop-Switchman {
    Get-Process -Name Switchman -ErrorAction SilentlyContinue | Stop-Process -Force
}

function Assert-Ok([string]$Step) {
    if ($LASTEXITCODE -ne 0) { throw "$Step failed (exit code $LASTEXITCODE)" }
}

if ($Uninstall) {
    Stop-Switchman
    if (Test-Path $cli) { Invoke-Cli 'unregister' }
    Remove-Item $shortcut -ErrorAction SilentlyContinue
    Remove-Item $Dest -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host 'Switchman uninstalled. Your configuration is still in' (Join-Path $env:LOCALAPPDATA 'Switchman')
    return
}

# Release: el zip trae los ejecutables junto a este script. Código fuente: se compila primero.
if (Test-Path (Join-Path $PSScriptRoot 'Switchman.exe')) {
    $publish = $PSScriptRoot
    $cliSource = Join-Path $PSScriptRoot 'switchman-cli.exe'
} else {
    Write-Host '==> Building the Rust CLI'
    cargo build --release --manifest-path (Join-Path $root 'Cargo.toml')
    Assert-Ok 'cargo build'

    Write-Host '==> Publishing the WinUI 3 app'
    $publish = Join-Path $root 'target\publish'
    Remove-Item $publish -Recurse -Force -ErrorAction SilentlyContinue
    dotnet publish (Join-Path $root 'ui\src\Switchman.App\Switchman.App.csproj') -c Release -r win-x64 -p:Platform=x64 --self-contained -o $publish
    Assert-Ok 'dotnet publish'
    $cliSource = Join-Path $root 'target\release\switchman-cli.exe'
}

Write-Host '==> Copying files to the install folder'
Stop-Switchman
New-Item -ItemType Directory -Force $Dest | Out-Null
Copy-Item (Join-Path $publish '*') $Dest -Recurse -Force
Copy-Item $cliSource $Dest -Force
# Se deja una copia del script en la instalación para poder desinstalar sin el zip.
Copy-Item $PSCommandPath (Join-Path $Dest 'install.ps1') -Force

Write-Host '==> Checking that the installed app starts'
$probe = Start-Process (Join-Path $Dest 'Switchman.exe') -PassThru
Start-Sleep -Seconds 5
if ($probe.HasExited) {
    throw "Switchman.exe closed on startup (exit code $($probe.ExitCode)); see %LOCALAPPDATA%\Switchman\switchman-ui.log"
}
Stop-Process -Id $probe.Id -Force

Write-Host '==> Creating the Start menu shortcut'
$shell = New-Object -ComObject WScript.Shell
$lnk = $shell.CreateShortcut($shortcut)
$lnk.TargetPath = Join-Path $Dest 'Switchman.exe'
$lnk.WorkingDirectory = $Dest
$lnk.Description = 'Switchman: routes links and removes tracking parameters'
$lnk.Save()

Write-Host '==> Registering Switchman as a browser'
Invoke-Cli 'register'

Write-Host "Done. Installed in $Dest. In the Windows Settings page that just opened, choose Switchman as your default browser."
