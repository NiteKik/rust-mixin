param(
    [string]$Toolchain = 'stable',
    [switch]$BuildOnly,
    [switch]$GameOnly,
    [ValidateSet('video-before', 'video-inside', 'video-after')][string]$Mod,
    [string]$OutputDirectory,
    [ValidateSet('Embedded', 'External')][string]$Bitcode
)
$ErrorActionPreference = 'Stop'
$utf8 = [System.Text.UTF8Encoding]::new($false)
[Console]::InputEncoding = $utf8
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$root = Split-Path -Parent $PSScriptRoot
$workspaceManifest = Join-Path $root 'Cargo.toml'
$fromSource = Test-Path -LiteralPath $workspaceManifest -PathType Leaf
if (($GameOnly -or $Mod -or $OutputDirectory) -and -not $fromSource) { throw '编译需要在源码目录运行' }
$artifacts = Join-Path $root 'artifacts\video-demo'
$mods = @(
    @{ Package = 'video-before'; Title = '修改函数前' },
    @{ Package = 'video-inside'; Title = '修改函数中' },
    @{ Package = 'video-after'; Title = '修改函数后' }
)
if ($OutputDirectory) {
    $OutputDirectory = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
}

function Build-DemoMod([string]$Package) {
    $modDirectory = Join-Path $modRoot $Package
    & "$PSScriptRoot\build-mod.ps1" -Toolchain $Toolchain -Package $Package -GameExecutable $exe -Output $modDirectory | Out-Null
    Write-Host "模组已编译 $modDirectory" -ForegroundColor Cyan
}

Push-Location -LiteralPath $root
try {
    $gameDirectory = $root
    if ($fromSource) {
        $metadata = & rustup run $Toolchain cargo metadata --locked --no-deps --format-version 1 --manifest-path $workspaceManifest
        if ($LASTEXITCODE -ne 0) { throw '无法读取 Cargo.toml 中的演示构建配置' }
        $config = ($metadata | ConvertFrom-Json).metadata.'video-demo'
        if (-not $OutputDirectory) {
            $configuredDirectory = $config.'output-directory'
            if ($configuredDirectory -isnot [string] -or [string]::IsNullOrWhiteSpace($configuredDirectory)) {
                throw '请在 Cargo.toml 的 workspace.metadata.video-demo 中填写 output-directory'
            }
            $OutputDirectory = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($configuredDirectory)
        }
        if (-not $Bitcode -and -not $Mod) {
            $Bitcode = $config.bitcode
            if ($Bitcode -notin @('embedded', 'external')) {
                throw 'Cargo.toml 的 workspace.metadata.video-demo.bitcode 必须是 embedded 或 external'
            }
        }
        $gameDirectory = $OutputDirectory
    }
    $exe = Join-Path $gameDirectory 'video-game.exe'
    $runtimeArtifacts = Join-Path $gameDirectory 'artifacts\video-demo'
    $modRoot = Join-Path $gameDirectory 'mods'

    if ($fromSource) {
        . "$PSScriptRoot\compiler.ps1"
        if ($Mod) {
            if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
                throw "请先在对应目录构建游戏：cargo build-game --output `"$gameDirectory`""
            }
            Build-DemoMod $Mod
            return
        }
        Write-Host '== 编译游戏 ==' -ForegroundColor Cyan
        $compiler = Get-CheckedCompiler -Toolchain $Toolchain
        Export-RetainedBitcode -Toolchain $Toolchain -Package video-game -Destination $artifacts -Stem retained_game
        $gameHash = (Get-FileHash -LiteralPath (Join-Path $artifacts 'retained_game.bc')).Hash.ToLowerInvariant()
        $manifest = [ordered]@{
            schema = 1
            rustc = $compiler
            llvm = @(21, 1, 3)
            triple = 'x86_64-pc-windows-msvc'
            cpu = 'x86-64'
            lowering_flags = @('cargo rustc --release', '--emit=llvm-bc,llvm-ir,link', '-Cno-prepopulate-passes', '-Clto=off', '--cfg=mixin_retained', '-Ctarget-cpu=x86-64')
            inputs = @([ordered]@{ file = 'retained_game.bc'; sha256 = $gameHash })
        }
        $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $artifacts 'manifest.json') -Encoding utf8
        $targetLib = & rustup run $Toolchain rustc --print target-libdir
        if ($LASTEXITCODE -ne 0) { throw '无法定位 Rust 标准库' }
        $stdDll = @(Get-ChildItem -LiteralPath $targetLib -Filter 'std-*.dll' -File)
        if ($stdDll.Count -ne 1) { throw '工具链中应有一份 Rust 标准库 DLL' }
        $savedStdDll = $env:MIXIN_DEMO_STD_DLL
        $features = @()
        if ($Bitcode -eq 'Embedded') { $features = @('--features', 'embedded-bitcode') }
        try {
            $env:MIXIN_DEMO_STD_DLL = $stdDll[0].Name
            & rustup run $Toolchain cargo rustc --profile video-demo --locked --quiet -p video-game --bin video-game @features -- -Cprefer-dynamic
            if ($LASTEXITCODE -ne 0) { throw '游戏编译失败' }
        } finally {
            $env:MIXIN_DEMO_STD_DLL = $savedStdDll
        }
        New-Item -ItemType Directory -Path $gameDirectory -Force | Out-Null
        $buildDirectory = Join-Path $root 'target\video-demo'
        if ($gameDirectory.TrimEnd('\', '/') -ne $buildDirectory) {
            Copy-Item -LiteralPath (Join-Path $buildDirectory 'video-game.exe') -Destination $gameDirectory -Force
        }
        Copy-Item -LiteralPath $stdDll[0].FullName -Destination $gameDirectory -Force
        Copy-Item -LiteralPath '.tools/llvm-21.1.3/bin/LLVM-C.dll' -Destination $gameDirectory -Force
        if ($Bitcode -eq 'External' -and $runtimeArtifacts -ne $artifacts) {
            New-Item -ItemType Directory -Path $runtimeArtifacts -Force | Out-Null
            Copy-Item -LiteralPath (Join-Path $artifacts 'retained_game.bc'), (Join-Path $artifacts 'manifest.json') -Destination $runtimeArtifacts -Force
        }
        Write-Host "游戏已编译 $exe" -ForegroundColor Cyan
    }

    if ($GameOnly) { return }

    if (-not $BuildOnly) {
        Write-Host '== 运行原版游戏 ==' -ForegroundColor Cyan
        & $exe
        if ($LASTEXITCODE -ne 0) { throw '原版游戏运行失败' }
    }

    if ($fromSource) {
        Write-Host '== 编译模组 ==' -ForegroundColor Cyan
        foreach ($demoMod in $mods) {
            Build-DemoMod $demoMod.Package
        }
    }

    if ($BuildOnly) { return }
    Write-Host '== 分别加载模组 ==' -ForegroundColor Cyan
    foreach ($demoMod in $mods) {
        Write-Host "--- $($demoMod.Title) ---" -ForegroundColor Yellow
        & $exe --mod (Join-Path $modRoot $demoMod.Package)
        if ($LASTEXITCODE -ne 0) { throw "加载模组后的游戏运行失败：$($demoMod.Title)" }
    }
} finally {
    Pop-Location
}

