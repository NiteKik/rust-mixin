param(
    [string]$Toolchain = 'stable',
    [ValidateSet('Embedded', 'External')][string]$Bitcode = 'Embedded',
    [ValidateSet('Embedded', 'External')][string]$Llvm = 'External'
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\compiler.ps1"
$projectRoot = Split-Path -Parent $PSScriptRoot
$artifactsRoot = Join-Path $projectRoot 'artifacts'
New-Item -ItemType Directory -Path $artifactsRoot -Force | Out-Null
Push-Location -LiteralPath $projectRoot
try {
    $compilerText = Get-CheckedCompiler -Toolchain $Toolchain
    Export-RetainedBitcode -Toolchain $Toolchain -Package llvm-fixture -Destination $artifactsRoot -Stem retained_game -SharedCrates mixin_sdk
    Export-RetainedBitcode -Toolchain $Toolchain -Package llvm-handler -Destination $artifactsRoot -Stem retained_mod
    # The runtime tests load these two fixtures directly. Other mods are exported by build-mod.ps1.
    Export-RetainedBitcode -Toolchain $Toolchain -Package llvm-half-mod -Destination $artifactsRoot -Stem half_mod
    Export-RetainedBitcode -Toolchain $Toolchain -Package llvm-parameter-mod -Destination $artifactsRoot -Stem parameter_mod
    $scalarArtifacts = Join-Path $artifactsRoot 'scalar'
    Export-RetainedBitcode -Toolchain $Toolchain -Package llvm-scalar-game -Destination $scalarArtifacts -Stem retained_game -SharedCrates mixin_sdk,example_return_types
    Export-RetainedBitcode -Toolchain $Toolchain -Package llvm-scalar-mod -Destination $scalarArtifacts -Stem retained_mod
    foreach ($directory in @($artifactsRoot, $scalarArtifacts)) {
        $inputs = foreach ($name in @('retained_game.bc','retained_mod.bc')) {
            $inputPath = Join-Path $directory $name
            [ordered]@{ file = $name; sha256 = (Get-FileHash -LiteralPath $inputPath -Algorithm SHA256).Hash.ToLowerInvariant() }
        }
        $manifest = [ordered]@{
            schema = 1
            rustc = $compilerText
            llvm = @(21,1,3)
            triple = 'x86_64-pc-windows-msvc'
            cpu = 'x86-64'
            lowering_flags = @('cargo rustc --release','--emit=llvm-bc,llvm-ir,link','-Cno-prepopulate-passes','-Clto=off','--cfg=mixin_retained','-Ctarget-cpu=x86-64')
            inputs = @($inputs)
        }
        $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $directory 'manifest.json') -Encoding utf8
    }
    $features = @('mixin-embedded-game/game-host')
    if ($Bitcode -eq 'Embedded') { $features += 'mixin-embedded-game/embedded-bitcode' }
    if ($Llvm -eq 'Embedded') { $features += 'mixin-embedded-game/embedded-llvm' }
    & rustup run $Toolchain cargo build --workspace --release --locked --features ($features -join ',')
    if ($LASTEXITCODE -ne 0) { throw 'Runner build failed' }
    if ($Llvm -eq 'External') {
        Copy-Item -LiteralPath '.tools/llvm-21.1.3/bin/LLVM-C.dll' -Destination 'target/release'
    }
    if ($Bitcode -eq 'External') {
        $runtimeArtifacts = 'target/release/artifacts'
        New-Item -ItemType Directory -Path $runtimeArtifacts -Force | Out-Null
        foreach ($file in @('retained_game.bc', 'manifest.json')) {
            Copy-Item -LiteralPath (Join-Path $artifactsRoot $file) -Destination $runtimeArtifacts
        }
    }
    Write-Output "Built game.exe (bitcode: $Bitcode, LLVM: $Llvm), CLI and benchmark."
} finally { Pop-Location }

