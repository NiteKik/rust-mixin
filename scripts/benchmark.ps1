param(
    [long]$Iterations = 16000000,
    [int]$Samples = 21,
    [int]$Runs = 3,
    [switch]$SkipBuild,
    [string]$ReportDirectory = 'reports/current'
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$reportsRoot = if ([IO.Path]::IsPathRooted($ReportDirectory)) { $ReportDirectory } else { Join-Path $projectRoot $ReportDirectory }
New-Item -ItemType Directory -Path $reportsRoot -Force | Out-Null
Push-Location -LiteralPath $projectRoot
try {
    if (-not $SkipBuild) { & "$PSScriptRoot\build.ps1" }
    $machine = [ordered]@{
        measured_at = (Get-Date -Format o)
        cpu = @(Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors)
        os = Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber
        iterations = $Iterations
        samples_per_comparison = $Samples
        process_runs = $Runs
        scheduling = 'Default Windows scheduler; no affinity pinning; hybrid CPU and background load can affect samples.'
    }
    $machine | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath "$reportsRoot\machine.json" -Encoding utf8
    $savedPath = $env:PATH
    try {
        $env:PATH = ''
        for ($runIndex = 1; $runIndex -le $Runs; $runIndex++) {
            $reportPath = Join-Path $reportsRoot ('run-{0:D2}.json' -f $runIndex)
            & '.\target\release\mixin-bench.exe' --iterations $Iterations --samples $Samples --report $reportPath
            if ($LASTEXITCODE -ne 0) { throw "Benchmark run $runIndex failed: $LASTEXITCODE" }
        }
    } finally { $env:PATH = $savedPath }
    $evidenceRoot = Join-Path $reportsRoot 'evidence'
    New-Item -ItemType Directory -Path $evidenceRoot -Force | Out-Null
    foreach ($name in @('manifest.json','call-index.json','logical-index.json','patch-plan.json','base.before-optimization.ll','base.optimized.ll','patched.before-optimization.ll','patched.optimized.ll')) {
        Copy-Item -LiteralPath (Join-Path $projectRoot "artifacts\$name") -Destination $evidenceRoot
    }
    $rows = @()
    for ($runIndex = 1; $runIndex -le $Runs; $runIndex++) {
        $result = Get-Content -LiteralPath (Join-Path $reportsRoot ('run-{0:D2}.json' -f $runIndex)) -Raw | ConvertFrom-Json
        foreach ($comparison in $result.comparisons) {
            for ($sampleIndex = 0; $sampleIndex -lt $comparison.samples; $sampleIndex++) {
                $rows += [pscustomobject]@{
                    run = $runIndex; workload = $comparison.name; sample = $sampleIndex
                    iterations = $comparison.iterations
                    aot_ns = $comparison.aot.ns_per_operation[$sampleIndex]
                    jit_ns = $comparison.jit.ns_per_operation[$sampleIndex]
                    jit_aot_ratio = $comparison.paired_ratios[$sampleIndex]
                }
            }
        }
    }
    $rows | Export-Csv -LiteralPath "$reportsRoot\samples.csv" -NoTypeInformation -Encoding utf8
} finally { Pop-Location }

