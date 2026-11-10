<#
.SYNOPSIS
  Instala (o desinstala) Switchman para el usuario actual, sin permisos de administrador.
.DESCRIPTION
  Compila el CLI de Rust y publica la UI WinUI3, copia ambos a -Dest, crea el acceso directo
  del menú Inicio y registra Switchman como navegador (abre Configuración para elegirlo).
  -Uninstall quita el registro, el acceso directo y la carpeta. Nunca borra la config del usuario
  (%LOCALAPPDATA%\Switchman).
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
    if ($p.ExitCode -ne 0) { throw "switchman-cli $Command terminó con $($p.ExitCode)" }
}

function Stop-Switchman {
    Get-Process -Name Switchman -ErrorAction SilentlyContinue | Stop-Process -Force
}

function Assert-Ok([string]$Step) {
    if ($LASTEXITCODE -ne 0) { throw "$Step falló (código $LASTEXITCODE)" }
}

if ($Uninstall) {
    Stop-Switchman
    if (Test-Path $cli) { Invoke-Cli 'unregister' }
    Remove-Item $shortcut -ErrorAction SilentlyContinue
    Remove-Item $Dest -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host 'Switchman desinstalado. La configuración sigue en' (Join-Path $env:LOCALAPPDATA 'Switchman')
    return
}

Write-Host '==> Compilando el CLI de Rust'
cargo build --release --manifest-path (Join-Path $root 'Cargo.toml')
Assert-Ok 'cargo build'

Write-Host '==> Publicando la UI WinUI3'
$publish = Join-Path $root 'target\publish'
Remove-Item $publish -Recurse -Force -ErrorAction SilentlyContinue
dotnet publish (Join-Path $root 'ui\src\Switchman.App\Switchman.App.csproj') -c Release -r win-x64 -p:Platform=x64 --self-contained -o $publish
Assert-Ok 'dotnet publish'

Write-Host '==> Copiando archivos a la carpeta de destino'
Stop-Switchman
New-Item -ItemType Directory -Force $Dest | Out-Null
Copy-Item (Join-Path $publish '*') $Dest -Recurse -Force
Copy-Item (Join-Path $root 'target\release\switchman-cli.exe') $Dest -Force

Write-Host '==> Comprobando que la UI instalada arranca'
$probe = Start-Process (Join-Path $Dest 'Switchman.exe') -PassThru
Start-Sleep -Seconds 5
if ($probe.HasExited) {
    throw "Switchman.exe se cerró al arrancar (código $($probe.ExitCode)); ver %LOCALAPPDATA%\Switchman\switchman-ui.log"
}
Stop-Process -Id $probe.Id -Force

Write-Host '==> Creando acceso directo en el menú Inicio'
$shell = New-Object -ComObject WScript.Shell
$lnk = $shell.CreateShortcut($shortcut)
$lnk.TargetPath = Join-Path $Dest 'Switchman.exe'
$lnk.WorkingDirectory = $Dest
$lnk.Description = 'Switchman: enruta enlaces y quita parámetros de rastreo'
$lnk.Save()

Write-Host '==> Registrando Switchman como navegador predeterminado'
Invoke-Cli 'register'

Write-Host "Listo. Instalado en $Dest. Elige Switchman como navegador en la ventana de Configuración que se abrió."
