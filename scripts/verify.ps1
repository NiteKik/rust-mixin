param([string]$Toolchain = 'stable')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    & rustup run $Toolchain cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & "$PSScriptRoot\build.ps1" -Toolchain $Toolchain
    & rustup run $Toolchain cargo test --workspace --release --locked --features mixin-embedded-game/embedded-bitcode
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    & rustup run $Toolchain cargo clippy --workspace --all-targets --release --locked --features mixin-embedded-game/embedded-bitcode -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & '.\target\release\mixin-bench.exe' --check-only --report reports/validation.json
    if ($LASTEXITCODE -ne 0) { throw 'AOT/JIT differential validation failed' }
    & "$PSScriptRoot\check-author-errors.ps1" -Toolchain $Toolchain
    & "$PSScriptRoot\check-external-mods.ps1"
    & "$PSScriptRoot\check-embedded-game.ps1"
    & "$PSScriptRoot\check-damage-plugin.ps1" -Toolchain $Toolchain
    & "$PSScriptRoot\check-loot-host.ps1" -Toolchain $Toolchain
    Write-Output 'LLVM Mixin workspace checks passed.'
} finally { Pop-Location }

