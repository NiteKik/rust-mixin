param(
    [string]$Toolchain = 'stable',
    [string]$GameDirectory = 'target/release'
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $game = (Resolve-Path -LiteralPath (Join-Path $GameDirectory 'game.exe')).Path
    $scratch = Join-Path $projectRoot "target/damage-plugin-$([guid]::NewGuid())"
    $hybrid = Join-Path $scratch 'hybrid'
    $plugin = Join-Path $scratch 'plugin'
    $mixin = Join-Path $scratch 'mixin'
    foreach ($directory in @($hybrid, $plugin, $mixin)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
    & "$PSScriptRoot\build-damage-plugin.ps1" -Toolchain $Toolchain -Output $hybrid
    Copy-Item -LiteralPath $game -Destination $scratch
    Copy-Item -LiteralPath (Join-Path $hybrid 'plugin.dll') -Destination $plugin
    foreach ($file in @('code.bc', 'mod.json')) {
        Copy-Item -LiteralPath (Join-Path $hybrid $file) -Destination $mixin
    }
    $exe = Join-Path $scratch 'game.exe'
    $savedPath = $env:PATH
    $savedLocalAppData = $env:LOCALAPPDATA
    Push-Location -LiteralPath $scratch
    try {
        $env:PATH = ''
        $env:LOCALAPPDATA = Join-Path $scratch 'cache'
        $output = & $exe --mod $plugin
        if ($LASTEXITCODE -ne 0 -or ($output -join "`n") -ne 'Damage: 8') {
            throw "Plugin-only AOT result mismatch: $output"
        }
        if (Test-Path -LiteralPath $env:LOCALAPPDATA) {
            throw 'Plugin-only game created an LLVM cache'
        }

        Copy-Item -LiteralPath (Join-Path (Split-Path -Parent $game) 'LLVM-C.dll') -Destination $scratch
        $output = & $exe --mod $mixin
        if ($LASTEXITCODE -ne 0 -or ($output -join "`n") -ne 'Damage: 20') {
            throw "Mixin-only result mismatch: $output"
        }
        $output = & $exe --mod $hybrid
        if ($LASTEXITCODE -ne 0 -or ($output -join "`n") -ne 'Damage: 10') {
            throw "Hybrid package result mismatch: $output"
        }
    } finally {
        Pop-Location
        $env:PATH = $savedPath
        $env:LOCALAPPDATA = $savedLocalAppData
    }
    $scratchDirectory = Get-Item -LiteralPath $scratch
    $targetDirectory = (Resolve-Path -LiteralPath (Join-Path $projectRoot 'target')).ProviderPath
    if ($scratchDirectory.Parent.FullName -eq $targetDirectory -and
        -not ($scratchDirectory.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        Remove-Item -LiteralPath $scratchDirectory.FullName -Recurse -Force
    }
    Write-Output 'Plugin-only AOT (8), Mixin-only JIT (20), and combined package (10) passed with an empty PATH.'
} finally { Pop-Location }
