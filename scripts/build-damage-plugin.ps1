param(
    [string]$Toolchain = 'stable',
    [string]$Output = ''
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $Output) { $Output = Join-Path $projectRoot 'artifacts/mods/llvm-damage-plugin' }
Push-Location -LiteralPath $projectRoot
try {
    $package = 'llvm-damage-plugin'
    & rustup run $Toolchain cargo rustc --release --locked --lib -p $package --crate-type cdylib
    if ($LASTEXITCODE -ne 0) { throw 'Damage plugin DLL build failed' }
    & "$PSScriptRoot\build-mod.ps1" -Toolchain $Toolchain -Package $package -Output $Output
    Copy-Item -LiteralPath 'target/release/llvm_damage_plugin.dll' -Destination (Join-Path $Output 'plugin.dll')
    Write-Output "Exported damage plugin and Mixin: $Output"
} finally { Pop-Location }
