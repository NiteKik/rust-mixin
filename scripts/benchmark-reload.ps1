param(
    [string]$Toolchain = 'stable',
    [ValidateRange(1, 10000)][int]$Cycles = 128,
    [ValidateRange(1, 20)][int]$Runs = 3,
    [string]$ReportDirectory = 'reports/reload',
    [switch]$SkipBuild,
    [switch]$Background
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/compiler.ps1"
Push-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)
try {
    if (-not $SkipBuild) {
        & "$PSScriptRoot/build.ps1" -Toolchain $Toolchain
    }
    $work = Join-Path (Get-Location) 'target/reload-bench'
    $large = Join-Path $work 'large'
    New-Item -ItemType Directory -Path $large, $ReportDirectory -Force | Out-Null
    [ordered]@{
        cpu = @(Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors)
        windows_version = [Environment]::OSVersion.Version.ToString()
        measured_cycles_per_run = $Cycles
        runs_per_component = $Runs
        background = [bool]$Background
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $ReportDirectory 'machine.json') -Encoding utf8

    # Build a deterministic larger component with the same drop-count behavior.
    $source = Join-Path $work 'large.rs'
    $lines = @(
        '#![no_std]'
        '#[mixin_sdk::targets("demo::loot")]'
        'pub mod loot {'
        '    fn base_count(level: i32) -> i32 { level + 1 }'
        '    pub fn drop_count(level: i32, lucky: bool, multiplier: f32, cap: u64) -> i32 {'
        '        let n = (base_count(level) as f32 * multiplier) as i32 + i32::from(lucky);'
        '        (n.max(0) as u64).min(cap) as i32'
        '    }'
    )
    for ($rule = 1; $rule -le 96; $rule++) {
        $lines += "    pub fn rule_$rule(seed: u64) -> u64 {"
        $lines += "        let mut value = seed.wrapping_add($rule);"
        $lines += '        for _ in 0..12 {'
        $lines += "            value = value.rotate_left($($rule % 23 + 1)).wrapping_mul($($rule * 2 + 1));"
        $lines += '            value ^= value >> 11;'
        $lines += '        }'
        $lines += '        value'
        $lines += '    }'
    }
    $lines += '}'
    $lines | Set-Content -LiteralPath $source -Encoding utf8

    $compiler = Get-CheckedCompiler -Toolchain $Toolchain
    $base = 'artifacts/scalar'
    $baseHash = Get-CheckedGameHash -Directory $base -Compiler $compiler
    $sdk = Get-CheckedCompiledSdk -Directory $base -GameHash $baseHash
    $sdkLibrary = (Get-Content -LiteralPath (Join-Path $sdk 'externs.txt') | Where-Object { $_ -like 'mixin_sdk=*' }) -replace '^mixin_sdk=', ''
    $gameBitcode = Join-Path $large 'retained_game.bc'
    $rustcArgs = @('--edition=2024', '--crate-type=lib', '--target=x86_64-pc-windows-msvc',
        '--cfg', 'mixin_retained', '--check-cfg=cfg(mixin_retained)', '--emit=llvm-bc',
        '-Copt-level=3', '-Cno-prepopulate-passes', '-Clto=off', '-Cpanic=abort', '-Ctarget-cpu=x86-64',
        '--extern', "mixin_sdk=$(Join-Path $sdk $sdkLibrary)", '-L', "dependency=$sdk", '-o', $gameBitcode)
    & rustup run $Toolchain rustc @rustcArgs $source
    if ($LASTEXITCODE -ne 0) { throw 'Could not build the larger reload fixture' }
    $manifest = Get-Content -LiteralPath (Join-Path $base 'manifest.json') -Raw | ConvertFrom-Json
    $manifest.inputs = @([ordered]@{
        file = 'retained_game.bc'
        sha256 = (Get-FileHash -LiteralPath $gameBitcode -Algorithm SHA256).Hash.ToLowerInvariant()
    })
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $large 'manifest.json') -Encoding utf8

    foreach ($case in @(@{ Name = 'scalar'; Artifacts = $base }, @{ Name = 'large'; Artifacts = $large })) {
        $bonus = Join-Path $work "$($case.Name)/bonus"
        $double = Join-Path $work "$($case.Name)/double"
        & "$PSScriptRoot/build-mod.ps1" -Toolchain $Toolchain -Package llvm-loot-bonus -GameArtifacts $case.Artifacts -Output $bonus
        & "$PSScriptRoot/build-mod.ps1" -Toolchain $Toolchain -Package llvm-loot-double -GameArtifacts $case.Artifacts -Output $double
        $savedPath = $env:PATH
        try {
            $env:PATH = ''
            for ($run = 1; $run -le $Runs; $run++) {
                $report = Join-Path $ReportDirectory ('{0}-{1:D2}.json' -f $case.Name, $run)
                $benchArgs = @('reload', '--cycles', $Cycles, '--artifacts', $case.Artifacts,
                    '--mod', $bonus, '--mod', $double, '--report', $report)
                if ($Background) { $benchArgs += '--background' }
                & '.\target\release\mixin-bench.exe' @benchArgs
                if ($LASTEXITCODE -ne 0) { throw "Reload benchmark failed for $($case.Name), run $run" }
            }
        } finally {
            $env:PATH = $savedPath
        }
    }
} finally {
    Pop-Location
}
