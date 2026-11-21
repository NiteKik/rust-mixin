param(
    [string]$Package = 'llvm-handler',
    [string]$Output = '',
    [string]$GameArtifacts = '',
    [string]$Toolchain = 'stable',
    [string]$ManifestPath = '',
    [string]$GameExecutable = ''
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\compiler.ps1"
$projectRoot = Split-Path -Parent $PSScriptRoot
if ($GameExecutable) {
    if ($GameArtifacts -or $ManifestPath) { throw '-GameExecutable is for demo builds without -GameArtifacts or -ManifestPath' }
    $GameExecutable = (Resolve-Path -LiteralPath $GameExecutable).ProviderPath
}
if ($ManifestPath) {
    if (-not $PSBoundParameters.ContainsKey('Package')) { throw '-Package is required with -ManifestPath' }
    $ManifestPath = (Resolve-Path -LiteralPath $ManifestPath).ProviderPath
    $buildRoot = Split-Path -Parent $ManifestPath
    if ($Output) { $Output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Output) }
    if ($GameArtifacts) { $GameArtifacts = (Resolve-Path -LiteralPath $GameArtifacts).ProviderPath }
} else {
    $buildRoot = $projectRoot
}
if (-not $Output) { $Output = Join-Path $buildRoot "artifacts\mods\$Package" }
if (-not $GameArtifacts) { $GameArtifacts = Join-Path $projectRoot 'artifacts' }
Push-Location -LiteralPath $buildRoot
try {
    $compiler = Get-CheckedCompiler -Toolchain $Toolchain
    if ($GameExecutable) {
        $gameHash = & $GameExecutable --game-hash
        if ($LASTEXITCODE -ne 0 -or $gameHash -cnotmatch '^[0-9a-f]{64}$' -or $gameHash -isnot [string]) {
            throw '无法读取游戏的 bitcode 校验值，请用 cargo build-game 重新构建游戏'
        }
    } else {
        $gameHash = Get-CheckedGameHash -Directory $GameArtifacts -Compiler $compiler
    }
    $compiledSdk = ''
    if ($ManifestPath -and (Test-Path -LiteralPath (Join-Path $GameArtifacts 'compiled-sdk/snapshot.json'))) {
        $compiledSdk = Get-CheckedCompiledSdk -Directory $GameArtifacts -GameHash $gameHash
        if (-not (Test-Path -LiteralPath (Join-Path $compiledSdk 'rustc-sdk.exe') -PathType Leaf)) {
            throw 'Compiled SDK is missing its compiler wrapper. Run export-mod-sdk.ps1 to create the author SDK.'
        }
    }
    # Only builds this mod and SDK dependencies. No game or runner rebuild.
    Export-RetainedBitcode -Toolchain $Toolchain -Package $Package -Destination $Output -Stem code -ManifestPath $ManifestPath -CompiledSdk $compiledSdk
    $manifest = [ordered]@{
        schema=1; rustc_commit='ded5c06cf21d2b93bffd5d884aa6e96934ee4234'
        llvm=@(21,1,3); triple='x86_64-pc-windows-msvc'; cpu='x86-64'
        expected_game_sha256=$gameHash
        code_sha256=(Get-FileHash -LiteralPath (Join-Path $Output 'code.bc') -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $Output 'mod.json') -Encoding utf8
    Write-Output "Exported independent mod: $Output"
} finally { Pop-Location }
