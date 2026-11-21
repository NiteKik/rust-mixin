param(
    [Parameter(Mandatory)][string]$Output,
    [string]$GameArtifacts = 'artifacts',
    [string]$Toolchain = 'stable'
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/compiler.ps1"
$projectRoot = Split-Path -Parent $PSScriptRoot
$compiler = Get-CheckedCompiler -Toolchain $Toolchain
$gameHash = Get-CheckedGameHash -Directory $GameArtifacts -Compiler $compiler
foreach ($directory in @('artifacts', 'crates/mixin-sdk/src', 'crates/mixin-macros/src', 'scripts')) {
    New-Item -ItemType Directory -Path (Join-Path $Output $directory) -Force | Out-Null
}
foreach ($file in @('manifest.json', 'retained_game.bc')) {
    Copy-Item -LiteralPath (Join-Path $GameArtifacts $file) -Destination (Join-Path $Output "artifacts/$file")
}
foreach ($file in @('crates/mixin-sdk/Cargo.toml', 'crates/mixin-sdk/src/lib.rs',
    'crates/mixin-macros/Cargo.toml', 'crates/mixin-macros/src/lib.rs',
    'scripts/build-mod.ps1', 'scripts/compiler.ps1')) {
    Copy-Item -LiteralPath (Join-Path $projectRoot $file) -Destination (Join-Path $Output $file)
}
$snapshot = Join-Path $GameArtifacts 'compiled-sdk'
if (Test-Path -LiteralPath $snapshot) {
    $null = Get-CheckedCompiledSdk -Directory $GameArtifacts -GameHash $gameHash
    $outputArtifacts = Join-Path $Output 'artifacts'
    $compiledSdk = Join-Path $outputArtifacts 'compiled-sdk'
    New-Item -ItemType Directory -Path $compiledSdk -Force | Out-Null
    $dependencies = Get-Content -LiteralPath (Join-Path $snapshot 'snapshot.json') -Raw | ConvertFrom-Json
    foreach ($inputFile in $dependencies.inputs) {
        Copy-Item -LiteralPath (Join-Path $snapshot $inputFile.file) -Destination $compiledSdk
    }
    Copy-Item -LiteralPath (Join-Path $snapshot 'snapshot.json') -Destination $compiledSdk
}
Write-Output "Exported mod SDK: $Output"
