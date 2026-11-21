param([string]$Toolchain = 'stable')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $release = Join-Path $projectRoot "target/loot-host-$([guid]::NewGuid().ToString('N'))"
    $artifacts = Join-Path $release 'artifacts'
    New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
    $exe = Join-Path $release 'loot-game.exe'
    Copy-Item -LiteralPath 'target/release/loot-game.exe' -Destination $exe
    Copy-Item -LiteralPath '.tools/llvm-21.1.3/bin/LLVM-C.dll' -Destination $release
    Copy-Item -LiteralPath '.tools/llvm-21.1.3/LLVM-LICENSE.txt' -Destination $release
    Copy-Item -LiteralPath @('artifacts/scalar/manifest.json', 'artifacts/scalar/retained_game.bc') -Destination $artifacts
    $trackedFiles = @($exe, (Join-Path $artifacts 'retained_game.bc'))
    $publishedHashes = (Get-FileHash -LiteralPath $trackedFiles).Hash -join ','
    $bonus = Join-Path $release 'bonus'
    $double = Join-Path $release 'double'
    $invalid = Join-Path $release 'invalid'
    $wrongType = Join-Path $release 'wrong-type'
    $tempDirectory = (Resolve-Path -LiteralPath ([IO.Path]::GetTempPath())).ProviderPath
    $authorRoot = Join-Path $tempDirectory "rust-mixin-author-$([guid]::NewGuid().ToString('N'))"
    $sdk = Join-Path $authorRoot 'game sdk'
    & "$PSScriptRoot/export-mod-sdk.ps1" -Toolchain $Toolchain -Output $sdk -GameArtifacts 'artifacts/scalar'
    Copy-Item -Recurse -LiteralPath 'tests/fixtures/return-types' -Destination $authorRoot
    foreach ($mod in @(
        @{ Package = 'llvm-loot-bonus'; Output = $bonus },
        @{ Package = 'llvm-loot-double'; Output = $double },
        @{ Package = 'llvm-loot-invalid'; Output = $invalid },
        @{ Package = 'llvm-loot-wrong-type'; Output = $wrongType }
    )) {
        $source = Join-Path $projectRoot "tests/fixtures/mods/loot-$((Split-Path -Leaf $mod.Output))"
        $member = Join-Path $authorRoot (Split-Path -Leaf $mod.Output)
        New-Item -ItemType Directory -Path (Join-Path $member 'src') -Force | Out-Null
        $sourceFile = Join-Path $member 'src/lib.rs'
        if ($mod.Output -eq $wrongType) {
            @'
#![no_std]

#[repr(C)]
pub struct Lookalike {
    x: f32,
    y: f32,
}

#[mixin_sdk::modify_return_result(id = "wrong_vector", target = "demo::physics::knockback")]
pub fn wrong_vector(value: Lookalike) -> Lookalike {
    value
}
'@ | Set-Content -LiteralPath $sourceFile -Encoding utf8
        } else {
            Copy-Item -LiteralPath (Join-Path $source 'src/lib.rs') -Destination $sourceFile
        }
        if ($mod.Output -eq $bonus) {
            @'

use example_return_types::LootResult;
use mixin_sdk::Vec2;

#[mixin_sdk::modify_return_result(id = "flip_y", target = "demo::physics::knockback")]
pub fn flip_y(old: Vec2) -> Vec2 {
    Vec2 { x: old.x, y: -old.y }
}

#[mixin_sdk::modify_return_result(id = "change_loot", target = "demo::returns::loot")]
pub fn change_loot(mut old: LootResult) -> LootResult {
    if let Ok(Some(loot)) = &mut old {
        loot.bytes[0] = 42;
    }
    old
}
'@ | Add-Content -LiteralPath $sourceFile -Encoding utf8
        }
        $cargoManifest = @"
[package]
name = "$($mod.Package)"
version = "0.1.0"
edition = "2024"

[lib]
name = "loot_rule"

[dependencies]
mixin-sdk = { path = "../game sdk/crates/mixin-sdk" }
example-return-types = { path = "../return-types" }
"@
        $cargoManifest | Set-Content -LiteralPath (Join-Path $member 'Cargo.toml') -Encoding utf8
        Copy-Item -LiteralPath (Join-Path $projectRoot 'Cargo.lock') -Destination $member
        Push-Location -LiteralPath $authorRoot
        try {
            $manifest = "$(Split-Path -Leaf $member)/Cargo.toml"
            & rustup run $Toolchain cargo generate-lockfile --offline --manifest-path $manifest
            if ($LASTEXITCODE -ne 0) { throw "Could not generate external lockfile: $manifest" }
            & "$sdk/scripts/build-mod.ps1" -Toolchain $Toolchain -Package $mod.Package -ManifestPath $manifest -Output $mod.Output
        } finally {
            Pop-Location
        }
    }
    Copy-Item -LiteralPath "$PSScriptRoot/check-loot-player.ps1" -Destination $release
    & "$release/check-loot-player.ps1"
    $cli = (Resolve-Path -LiteralPath 'target/release/mixin-cli.exe').Path
    $arguments = @('--dll', (Join-Path $release 'LLVM-C.dll'), '--artifacts', $artifacts)
    $vectorReport = Join-Path $release 'external-vec2.json'
    & $cli @arguments --mod $bonus --call 'demo::physics::knockback' --arg 2 --report $vectorReport
    if ($LASTEXITCODE -ne 0) { throw 'External Vec2 mod failed' }
    $vector = Get-Content -LiteralPath $vectorReport -Raw | ConvertFrom-Json
    if ($vector.base_value.x -ne 2 -or $vector.base_value.y -ne -1 -or
        $vector.patched_value.x -ne 2 -or $vector.patched_value.y -ne 1) {
        throw 'External Vec2 mod returned an unexpected value'
    }
    $sharedReport = Join-Path $release 'external-shared-type.json'
    & $cli @arguments --mod $bonus --call 'demo::returns::loot_first_byte' --arg 7 --report $sharedReport
    if ($LASTEXITCODE -ne 0) { throw 'External game-defined return type mod failed' }
    $shared = Get-Content -LiteralPath $sharedReport -Raw | ConvertFrom-Json
    if ($shared.base_value -ne 7 -or $shared.patched_value -ne 42) {
        throw 'External game-defined return type mod returned an unexpected value'
    }
    $wrongTypeReport = Join-Path $release 'external-wrong-type.json'
    try {
        $ErrorActionPreference = 'Continue'
        & $cli @arguments --mod $wrongType --call 'demo::physics::knockback' --arg 2 --report $wrongTypeReport 2> (Join-Path $release 'external-wrong-type.stderr.log')
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = 'Stop'
    }
    $failure = Get-Content -LiteralPath $wrongTypeReport -Raw | ConvertFrom-Json
    if ($exitCode -eq 0 -or $failure.diagnostic.handler_id -ne 'wrong_vector' -or
        $failure.diagnostic.message -notmatch 'typed return mismatch') {
        throw 'Expected the loader to reject a distinct Rust type with the same layout'
    }
    Write-Output 'External typed returns verified: SDK Vec2, game-defined LootResult, and rejected lookalike.'
    $savedPath = $env:PATH
    try {
        $env:PATH = ''
        Push-Location -LiteralPath $release
        try {
            $commands = @('drop', 'prepare bonus', 'prepare double', 'drop', 'wait', 'drop',
                'prepare double', 'drop', 'wait', 'drop', 'prepare bonus|invalid',
                'drop', 'wait', 'drop', 'prepare', 'drop', 'wait', 'drop')
            $expected = @('Loot: 3, total: 3', 'Loot: 3, total: 6', 'Loot: 4, total: 10',
                'Loot: 4, total: 14', 'Loot: 6, total: 20', 'Loot: 6, total: 26',
                'Loot: 6, total: 32', 'Loot: 6, total: 38', 'Loot: 3, total: 41')
            $total = 41
            for ($cycle = 0; $cycle -lt 32; $cycle++) {
                $commands += @('mods bonus|double', 'drop', 'mods', 'drop')
                $total += 8
                $expected += "Loot: 8, total: $total"
                $total += 3
                $expected += "Loot: 3, total: $total"
            }
            $diagnostic = Join-Path $release 'hot-reload.stderr.log'
            try {
                $ErrorActionPreference = 'Continue'
                $output = $commands + 'quit' | & $exe --interactive 2>&1
                $exitCode = $LASTEXITCODE
            } finally {
                $ErrorActionPreference = 'Stop'
            }
            $transcript = @($output | Where-Object { $_ -is [string] })
            $messages = @($output | Where-Object { $_ -is [System.Management.Automation.ErrorRecord] } | ForEach-Object { $_.ToString() })
            $transcript | Set-Content -LiteralPath (Join-Path $release 'hot-reload.stdout.log') -Encoding utf8
            $messages | Set-Content -LiteralPath $diagnostic -Encoding utf8
            $loot = @($transcript | Where-Object { $_ -like 'Loot: *' })
            if ($exitCode -ne 0 -or ($loot -join "`n") -ne ($expected -join "`n") -or
                @($transcript | Where-Object { $_ -like 'Mods applied: *' }).Count -ne 67 -or
                $messages.Count -ne 2 -or $messages[0] -notmatch 'a reload is already pending' -or
                $messages[1] -notmatch 'invalid: wrong_loot_position: expected 2 original matches.+found 1') {
                throw "Unexpected hot reload result. See $release"
            }
        } finally {
            Pop-Location
        }
    } finally {
        $env:PATH = $savedPath
    }
    if (((Get-FileHash -LiteralPath $trackedFiles).Hash -join ',') -ne $publishedHashes) {
        throw 'The published game executable or its bitcode changed during mod builds'
    }
    $authorDirectory = Get-Item -LiteralPath $authorRoot
    if ($authorDirectory.Parent.FullName.TrimEnd([char[]]'\/') -eq $tempDirectory.TrimEnd([char[]]'\/') -and
        -not ($authorDirectory.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        Remove-Item -LiteralPath $authorDirectory.FullName -Recurse -Force
    }
    Write-Output 'Independent loot host verified with an empty PATH: 3, 4, 6, 3. Invalid mod rejected.'
    Write-Output 'Background hot reload verified: calls keep using the published rules during preparation, then 32 cycles with two mods and restoration.'
} finally { Pop-Location }
