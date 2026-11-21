param(
    [string]$GameDirectory = 'target/release',
    [string]$ModDirectory = 'artifacts/mods/llvm-half-mod',
    [ValidateSet('Embedded', 'External')][string]$Bitcode = 'Embedded',
    [ValidateSet('Embedded', 'External')][string]$Llvm = 'External'
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $game = (Resolve-Path -LiteralPath (Join-Path $GameDirectory 'game.exe')).Path
    $source = Split-Path -Parent $game
    $mod = (Resolve-Path -LiteralPath $ModDirectory).Path
    $scratch = Join-Path $projectRoot "target/embedded-game-$([guid]::NewGuid())"
    New-Item -ItemType Directory -Path $scratch | Out-Null
    Copy-Item -LiteralPath $game -Destination $scratch
    $embeddedGame = Join-Path $scratch 'game.exe'

    Push-Location -LiteralPath $scratch
    $savedPath = $env:PATH
    $savedLocalAppData = $env:LOCALAPPDATA
    try {
        $env:PATH = ''
        $env:LOCALAPPDATA = Join-Path $scratch 'cache'
        $base = & $embeddedGame
        if ($LASTEXITCODE -ne 0 -or ($base -join "`n") -ne 'Damage: 16') {
            throw "Standalone AOT game result mismatch: $base"
        }
        if (Test-Path -LiteralPath $env:LOCALAPPDATA) {
            throw 'Unmodified game created an LLVM cache'
        }

        if ($Llvm -eq 'External') {
            Copy-Item -LiteralPath (Join-Path $source 'LLVM-C.dll') -Destination $scratch
        }
        if ($Bitcode -eq 'External') {
            $artifacts = Join-Path $scratch 'artifacts'
            New-Item -ItemType Directory -Path $artifacts | Out-Null
            foreach ($file in @('retained_game.bc', 'manifest.json')) {
                Copy-Item -LiteralPath (Join-Path $source "artifacts/$file") -Destination $artifacts
            }
        }
        $modFolder = Join-Path $scratch 'half mod'
        New-Item -ItemType Directory -Path $modFolder | Out-Null
        foreach ($file in @('mod.json', 'code.bc')) {
            Copy-Item -LiteralPath (Join-Path $mod $file) -Destination $modFolder
        }
        $patched = & $embeddedGame --mod 'half mod'
        if ($LASTEXITCODE -ne 0 -or ($patched -join "`n") -ne 'Damage: 8') {
            throw "JIT game result mismatch: $patched"
        }
        if ($Llvm -eq 'Embedded') {
            $dll = Join-Path $projectRoot '.tools/llvm-21.1.3/bin/LLVM-C.dll'
            $expectedHash = (Get-FileHash -LiteralPath $dll).Hash
            $cachedDll = Join-Path $env:LOCALAPPDATA "RustMixin/llvm/$($expectedHash.ToLowerInvariant())/LLVM-C.dll"
            if ((Get-FileHash -LiteralPath $cachedDll).Hash -ne $expectedHash) {
                throw 'Embedded LLVM cache hash mismatch'
            }
            $cachedFile = [IO.File]::OpenWrite($cachedDll)
            try {
                $cachedFile.WriteByte(0)
            } finally {
                $cachedFile.Dispose()
            }
            $repaired = & $embeddedGame --mod 'half mod'
            if ($LASTEXITCODE -ne 0 -or ($repaired -join "`n") -ne 'Damage: 8') {
                throw "JIT game failed with a damaged LLVM cache: $repaired"
            }
            if ((Get-FileHash -LiteralPath $cachedDll).Hash -ne $expectedHash) {
                throw 'Embedded LLVM did not repair the cached DLL'
            }
            $license = & $embeddedGame --llvm-license
            if ($LASTEXITCODE -ne 0 -or ($license -join "`n") -notmatch 'Apache License') {
                throw 'Embedded LLVM license is unavailable'
            }
        }
    } finally {
        $env:PATH = $savedPath
        $env:LOCALAPPDATA = $savedLocalAppData
        Pop-Location
    }
    $scratchDirectory = Get-Item -LiteralPath $scratch
    $targetDirectory = (Resolve-Path -LiteralPath (Join-Path $projectRoot 'target')).ProviderPath
    if ($scratchDirectory.Parent.FullName -eq $targetDirectory -and
        -not ($scratchDirectory.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        Remove-Item -LiteralPath $scratchDirectory.FullName -Recurse -Force
    }
    Write-Output "Standalone AOT (16) and modded JIT (8) passed with bitcode=$Bitcode, LLVM=$Llvm and an empty PATH."
} finally { Pop-Location }
