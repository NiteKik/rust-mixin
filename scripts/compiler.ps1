function Get-CheckedCompiler {
    param([string]$Toolchain)
    $compiler = & rustup run $Toolchain rustc -vV
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect rustc' }
    $text = $compiler -join "`n"
    if ($text -notmatch 'commit-hash: ded5c06cf21d2b93bffd5d884aa6e96934ee4234' -or
        $text -notmatch 'LLVM version: 21\.1\.3' -or
        $text -notmatch 'host: x86_64-pc-windows-msvc') {
        throw 'Requires rustc 1.92.0 ded5c06cf / LLVM 21.1.3 / Windows x64. Use -Toolchain 1.92.0 if stable moved.'
    }
    return $text
}

function Export-RetainedBitcode {
    param(
        [string]$Toolchain, [string]$Package, [string]$Destination, [string]$Stem,
        [string]$ManifestPath, [string[]]$SharedCrates = @(), [string]$CompiledSdk
    )
    $targetDirectory = Join-Path (Get-Location) "target\retained-$Package"
    if ($CompiledSdk) {
        $snapshotHash = (Get-FileHash -LiteralPath (Join-Path $CompiledSdk 'snapshot.json') -Algorithm SHA256).Hash.ToLowerInvariant()
        # Keep the build path short enough for the Windows linker.
        $targetDirectory = Join-Path (Get-Location) "target\sdk-$($snapshotHash.Substring(0, 16))"
    }
    $flags = @('--emit=llvm-bc,llvm-ir,link','-C','no-prepopulate-passes','-C','lto=off','--cfg','mixin_retained','-C','target-cpu=x86-64')
    $manifestArgs = @()
    if ($ManifestPath) { $manifestArgs = @('--manifest-path', $ManifestPath) }
    $packageId = & rustup run $Toolchain cargo pkgid --locked -p $Package @manifestArgs
    if ($LASTEXITCODE -ne 0) { throw "Cannot resolve Cargo package: $Package" }
    $cargoArgs = @('--quiet','--locked','--release','--lib','-p',$Package,'--target-dir',$targetDirectory,'--message-format=json')
    if ($ManifestPath) {
        $cargoArgs += $manifestArgs + @('--target','x86_64-pc-windows-msvc')
        $flags += @('-C','panic=abort','-C','codegen-units=1','--check-cfg=cfg(mixin_retained)')
    }
    $savedWrapper = $env:RUSTC_WORKSPACE_WRAPPER
    $savedSdk = $env:RUST_MIXIN_COMPILED_SDK
    try {
        if ($CompiledSdk) {
            $env:RUSTC_WORKSPACE_WRAPPER = Join-Path $CompiledSdk 'rustc-sdk.exe'
            $env:RUST_MIXIN_COMPILED_SDK = $CompiledSdk
        }
        $lines = & rustup run $Toolchain cargo rustc @cargoArgs -- @flags
        $buildExit = $LASTEXITCODE
    } finally {
        $env:RUSTC_WORKSPACE_WRAPPER = $savedWrapper
        $env:RUST_MIXIN_COMPILED_SDK = $savedSdk
    }
    $messages = @($lines | ForEach-Object { $_ | ConvertFrom-Json })
    $messages | Where-Object reason -eq 'compiler-message' | ForEach-Object {
        if ($_.message.rendered) { $Host.UI.Write($_.message.rendered) }
    }
    if ($buildExit -ne 0) { throw "Retained build failed: $Package" }
    $records = @($messages | Where-Object {
        $_.reason -eq 'compiler-artifact' -and $_.package_id -eq $packageId -and
        ($_.target.kind -contains 'lib' -or $_.target.kind -contains 'rlib')
    })
    if ($records.Count -ne 1) { throw "Ambiguous Cargo artifact: $Package" }
    $rmeta = @($records[0].filenames | Where-Object { $_.EndsWith('.rmeta') })
    if ($rmeta.Count -ne 1) { throw "Missing rmeta identity: $Package" }
    $baseName = [IO.Path]::GetFileNameWithoutExtension($rmeta[0]).Substring(3)
    $directory = Split-Path -Parent $rmeta[0]
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    foreach ($extension in @('bc','ll')) {
        Copy-Item -LiteralPath (Join-Path $directory "$baseName.$extension") -Destination (Join-Path $Destination "$Stem.$extension")
    }
    if ($SharedCrates.Count) {
        $dependencies = @($messages | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.package_id -ne $packageId })
        Export-CompiledDependencies -Dependencies $dependencies -Names $SharedCrates -Directory $Destination -Bitcode "$Stem.bc" -Toolchain $Toolchain
    }
}

function Export-CompiledDependencies {
    param([object[]]$Dependencies, [string[]]$Names, [string]$Directory, [string]$Bitcode, [string]$Toolchain)
    $files = @($dependencies | ForEach-Object filenames | Where-Object { $_ -match '\.(rlib|dll)$' } | Sort-Object -Unique)
    $externs = foreach ($name in $Names) {
        $matches = @($dependencies | Where-Object { $_.target.name -ceq $name })
        $library = @($matches | ForEach-Object filenames | Where-Object { $_.EndsWith('.rlib') })
        if ($library.Count -ne 1) { throw "Expected one compiled dependency for '$name'" }
        "$name=$([IO.Path]::GetFileName($library[0]))"
    }
    $snapshot = Join-Path $Directory 'compiled-sdk'
    New-Item -ItemType Directory -Path $snapshot -Force | Out-Null
    $inputs = foreach ($file in $files) {
        $name = [IO.Path]::GetFileName($file)
        Copy-Item -LiteralPath $file -Destination (Join-Path $snapshot $name)
        [ordered]@{ file = $name; sha256 = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant() }
    }
    $externs | Set-Content -LiteralPath (Join-Path $snapshot 'externs.txt') -Encoding ascii
    & rustup run $Toolchain rustc --edition=2024 --target x86_64-pc-windows-msvc -C opt-level=2 -o (Join-Path $snapshot 'rustc-sdk.exe') (Join-Path $PSScriptRoot 'rustc-sdk.rs')
    if ($LASTEXITCODE -ne 0) { throw 'Could not build the SDK rustc wrapper' }
    foreach ($name in @('externs.txt', 'rustc-sdk.exe')) {
        $inputs += [ordered]@{ file = $name; sha256 = (Get-FileHash -LiteralPath (Join-Path $snapshot $name) -Algorithm SHA256).Hash.ToLowerInvariant() }
    }
    $manifest = [ordered]@{
        game_sha256 = (Get-FileHash -LiteralPath (Join-Path $Directory $Bitcode) -Algorithm SHA256).Hash.ToLowerInvariant()
        inputs = @($inputs)
    }
    $manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $snapshot 'snapshot.json') -Encoding utf8
}

function Get-CheckedCompiledSdk {
    param([string]$Directory, [string]$GameHash)
    $snapshot = Join-Path $Directory 'compiled-sdk'
    $manifest = Get-Content -LiteralPath (Join-Path $snapshot 'snapshot.json') -Raw | ConvertFrom-Json
    if ($manifest.game_sha256 -ne $GameHash) { throw 'Compiled SDK corresponds to a different game build' }
    foreach ($inputFile in $manifest.inputs) {
        $name = [string]$inputFile.file
        if ($name -notmatch '^[A-Za-z0-9_-]+\.(rlib|dll|txt)$' -and $name -cne 'rustc-sdk.exe') {
            throw "Invalid compiled SDK filename: $name"
        }
        $path = Join-Path $snapshot $name
        if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $inputFile.sha256) {
            throw "Compiled SDK file hash mismatch: $name"
        }
    }
    return $snapshot
}

function Get-CheckedGameHash {
    param([string]$Directory, [string]$Compiler)
    $manifest = Get-Content -LiteralPath (Join-Path $Directory 'manifest.json') -Raw | ConvertFrom-Json
    if ($manifest.rustc -ne $Compiler -or $manifest.cpu -ne 'x86-64' -or
        $manifest.triple -ne 'x86_64-pc-windows-msvc' -or $manifest.schema -ne 1 -or
        ($manifest.llvm -join '.') -ne '21.1.3') {
        throw 'Game SDK compiler identity mismatch'
    }
    $gameHash = (Get-FileHash -LiteralPath (Join-Path $Directory 'retained_game.bc') -Algorithm SHA256).Hash.ToLowerInvariant()
    $declaredHash = @($manifest.inputs | Where-Object file -eq 'retained_game.bc')
    if ($declaredHash.Count -ne 1 -or $gameHash -ne $declaredHash[0].sha256) { throw 'Game SDK payload mismatch' }
    return $gameHash
}
