param([switch]$RequireNoRust)
$ErrorActionPreference = 'Stop'
if ($RequireNoRust -and (Get-Command -Name cargo.exe,rustc.exe,rustup.exe -ErrorAction SilentlyContinue)) {
    throw 'Rust tools are visible on PATH. Run this check in the clean Windows test environment.'
}
$exe = Join-Path $PSScriptRoot 'loot-game.exe'
$bonus = Join-Path $PSScriptRoot 'bonus'
$double = Join-Path $PSScriptRoot 'double'
$invalid = Join-Path $PSScriptRoot 'invalid'
$savedPath = $env:PATH
try {
    $env:PATH = ''
    foreach ($case in @(
        @{ Arguments = @(); Expected = 'Loot: 3' },
        @{ Arguments = @('--plan', '--mod', $bonus); Expected = "bonus_loot -> demo::loot::drop_count (1 match)`nchange_loot -> demo::returns::loot (1 match)`nflip_y -> demo::physics::knockback (1 match)" },
        @{ Arguments = @('--mod', $bonus); Expected = 'Loot: 4' },
        @{ Arguments = @('--mod', $double); Expected = 'Loot: 6' },
        @{ Arguments = @(); Expected = 'Loot: 3' }
    )) {
        $arguments = $case.Arguments
        $output = & $exe @arguments
        if ($LASTEXITCODE -ne 0 -or ($output -join "`n") -ne $case.Expected) {
            throw "Expected '$($case.Expected)', got '$output'"
        }
    }
    $diagnostic = Join-Path $PSScriptRoot 'invalid-startup.log'
    try {
        $ErrorActionPreference = 'Continue'
        $output = & $exe --mod $invalid 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = 'Stop'
    }
    $message = @($output | Where-Object { $_ -is [System.Management.Automation.ErrorRecord] }) -join "`n"
    $message | Set-Content -LiteralPath $diagnostic -Encoding utf8
    if ($exitCode -eq 0 -or -not $message.Contains($invalid) -or
        $message -notmatch 'handler wrong_loot_position targeting demo::loot::drop_count' -or
        $message -notmatch 'expected 2 original matches.+found 1') {
        throw "Invalid selector did not produce the expected diagnostic: $message"
    }
    Write-Output 'Player startup verified with an empty PATH: 3, 4, 6, 3. Invalid mod rejected.'
} finally {
    $env:PATH = $savedPath
}
