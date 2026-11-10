<#
.SYNOPSIS
  Arma el zip del release para Windows x64: target\dist\Switchman-<versión>-win-x64.zip
.DESCRIPTION
  Compila el CLI, publica la UI autocontenida y junta todo con install.ps1 y los README.
  La versión sale de Cargo.toml ([workspace.package] version).
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$version = (Select-String (Join-Path $root 'Cargo.toml') -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$name = "Switchman-$version-win-x64"
$stage = Join-Path $root "target\dist\$name"
$zip = Join-Path $root "target\dist\$name.zip"

cargo build --release --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'cargo build falló' }
Remove-Item $stage, $zip -Recurse -Force -ErrorAction SilentlyContinue
dotnet publish (Join-Path $root 'ui\src\Switchman.App\Switchman.App.csproj') -c Release -r win-x64 -p:Platform=x64 --self-contained -p:Version=$version -o $stage
if ($LASTEXITCODE -ne 0) { throw 'dotnet publish falló' }

Copy-Item (Join-Path $root 'target\release\switchman-cli.exe') $stage
Copy-Item (Join-Path $PSScriptRoot 'install.ps1') $stage
Copy-Item (Join-Path $root 'README.md'), (Join-Path $root 'README.es.md') $stage
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal
Write-Host "Listo: $zip ($([math]::Round((Get-Item $zip).Length / 1MB)) MB)"
