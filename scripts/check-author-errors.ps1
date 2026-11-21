param([string]$Toolchain = 'stable')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$scratch = Join-Path $projectRoot 'target\author-errors'
New-Item -ItemType Directory -Path $scratch -Force | Out-Null
$cases = @(
    @{
        name = 'borrow'; expected = 'E0597'; source = @'
#![no_std]
#[mixin_sdk::modify_call_result(id="bad_borrow",target="g::a",callee="g::b")]
pub fn adjust(value:f64)->f64 {
    let reference: &f64;
    { let temporary=value; reference=&temporary; }
    *reference
}
'@
    },
    @{
        name = 'ordinal'; expected = 'ordinal must be smaller'; source = @'
#![no_std]
#[mixin_sdk::modify_call_result(id="bad_ordinal",target="g::a",callee="g::b",ordinal=2,require_matches=1)]
pub fn adjust(value:f64)->f64 { value }
'@
    },
    @{
        name = 'head_selector'; expected = 'not valid for this operation'; source = @'
#![no_std]
#[mixin_sdk::modify_head(id="bad_head",target="g::a",ordinal=1)]
pub fn adjust(value:f64)->f64 { value }
'@
    }
)
foreach ($case in $cases) {
    $source = Join-Path $scratch ($case.name + '.rs')
    $case.source | Set-Content -LiteralPath $source -Encoding utf8
    $previousPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $diagnostics = & rustup run $Toolchain rustc $source --crate-name author_error --crate-type lib --edition=2024 --cfg mixin_retained '--check-cfg=cfg(mixin_retained)' --extern "mixin_sdk=$projectRoot\target\release\libmixin_sdk.rlib" -L "dependency=$projectRoot\target\release\deps" --out-dir $scratch 2>&1
        $compilerExit = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previousPreference }
    $text = $diagnostics -join "`n"
    $text | Set-Content -LiteralPath (Join-Path $scratch ($case.name + '.txt')) -Encoding utf8
    if ($compilerExit -eq 0 -or $text -notmatch [regex]::Escape($case.expected)) {
        throw "Expected author compile failure missing: $($case.name): $text"
    }
    Write-Output "Expected compiler rejection passed: $($case.name)"
}
