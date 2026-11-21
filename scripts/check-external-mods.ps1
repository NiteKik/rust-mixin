$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $exeHash = (Get-FileHash -LiteralPath 'target/release/mixin-cli.exe' -Algorithm SHA256).Hash
    $gameHash = (Get-FileHash -LiteralPath 'artifacts/retained_game.bc' -Algorithm SHA256).Hash
    & "$PSScriptRoot\build-mod.ps1" -Package llvm-handler
    & "$PSScriptRoot\build-mod.ps1" -Package llvm-half-mod
    & "$PSScriptRoot\build-mod.ps1" -Package llvm-return-mod
    & "$PSScriptRoot\build-mod.ps1" -Package llvm-parameter-mod
    & "$PSScriptRoot\build-mod.ps1" -Package llvm-head-mod
    if ($exeHash -ne (Get-FileHash -LiteralPath 'target/release/mixin-cli.exe' -Algorithm SHA256).Hash -or
        $gameHash -ne (Get-FileHash -LiteralPath 'artifacts/retained_game.bc' -Algorithm SHA256).Hash) {
        throw 'Independent mod build unexpectedly changed the game or runner'
    }
    $cases = @(
        @{ name='external-adjust'; mods=@('llvm-handler'); expected=9.5 },
        @{ name='external-half'; mods=@('llvm-half-mod'); expected=8.0 },
        @{ name='external-combined'; mods=@('llvm-handler','llvm-half-mod'); expected=4.75 },
        @{ name='external-reversed'; mods=@('llvm-half-mod','llvm-handler'); expected=4.0 },
        @{ name='external-return'; mods=@('llvm-return-mod'); expected=18.0 },
        @{ name='external-parameter'; mods=@('llvm-parameter-mod'); expected=20.0 },
        @{ name='external-head'; mods=@('llvm-head-mod'); expected=20.0 }
    )
    $savedPath=$env:PATH
    try {
        $env:PATH=''
        foreach ($case in $cases) {
            $report = "reports/current/$($case.name).json"
            $arguments=@('--report',$report,'--call','demo::combat::apply_damage')
            foreach ($mod in $case.mods) { $arguments += @('--mod',"artifacts/mods/$mod") }
            & '.\target\release\mixin-cli.exe' @arguments
            if ($LASTEXITCODE -ne 0) { throw "External mod process failed: $($case.name)" }
            $result=Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
            if ($result.base_value -ne 16.0 -or $result.base_unchanged -ne 16.0 -or
                $result.patched_value -ne $case.expected -or -not $result.empty_path) {
                throw "External mod result mismatch: $($case.name)"
            }
        }
    } finally { $env:PATH=$savedPath }
    Write-Output 'Independent export, unchanged game/runner hashes, and seven empty-PATH processes passed.'
} finally { Pop-Location }
