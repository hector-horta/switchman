<#
.SYNOPSIS
  Arma el release para Windows x64 en target\dist: el instalador Switchman-<versión>-win-x64-setup.exe
  y el zip Switchman-<versión>-win-x64.zip.
.DESCRIPTION
  Compila el CLI y publica la UI autocontenida. Con esa carpeta, Inno Setup (ISCC) arma el instalador;
  después se le suman install.ps1 y los README para el zip. ISCC se busca en $env:ISCC o en las rutas
  de instalación habituales de Inno Setup 6.
  La versión sale de Cargo.toml ([workspace.package] version).
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$version = (Select-String (Join-Path $root 'Cargo.toml') -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$name = "Switchman-$version-win-x64"
$stage = Join-Path $root "target\dist\$name"
$zip = Join-Path $root "target\dist\$name.zip"
$setup = Join-Path $root "target\dist\$name-setup.exe"
$iscc = @($env:ISCC, "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
    Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw 'ISCC.exe (Inno Setup 6) not found; install it or set $env:ISCC' }

cargo build --release --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
Remove-Item $stage, $zip, $setup -Recurse -Force -ErrorAction SilentlyContinue
dotnet publish (Join-Path $root 'ui\src\Switchman.App\Switchman.App.csproj') -c Release -r win-x64 -p:Platform=x64 --self-contained -p:Version=$version -o $stage
if ($LASTEXITCODE -ne 0) { throw 'dotnet publish failed' }

Copy-Item (Join-Path $root 'target\release\switchman-cli.exe') $stage
& $iscc /Q "/DAppVersion=$version" /DArch=x64 "/DSourceDir=$stage" (Join-Path $root 'installer\switchman.iss')
if ($LASTEXITCODE -ne 0) { throw 'ISCC failed' }

Copy-Item (Join-Path $PSScriptRoot 'install.ps1') $stage
Copy-Item (Join-Path $root 'README.md'), (Join-Path $root 'README.es.md') $stage
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal
foreach ($file in $setup, $zip) { Write-Host "Done: $file ($([math]::Round((Get-Item $file).Length / 1MB)) MB)" }
