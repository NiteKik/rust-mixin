param([string]$Toolchain = 'stable')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $root 'dist'

& "$PSScriptRoot\video-demo.ps1" -Toolchain $Toolchain -BuildOnly -OutputDirectory $destination
$sysroot = & rustup run $Toolchain rustc --print sysroot
if ($LASTEXITCODE -ne 0) { throw '无法定位 Rust 标准库许可证' }
$rustDocs = Join-Path $sysroot 'share/doc/rust'
$rustLicenses = Join-Path $destination 'rust-licenses'

New-Item -ItemType Directory -Path (Join-Path $destination 'scripts') -Force | Out-Null
Copy-Item -LiteralPath @(
    (Join-Path $root 'demo.cmd'),
    (Join-Path $root '.tools/llvm-21.1.3/LLVM-LICENSE.txt')
) -Destination $destination
Copy-Item -LiteralPath "$PSScriptRoot\video-demo.ps1" -Destination (Join-Path $destination 'scripts')
New-Item -ItemType Directory -Path $rustLicenses -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $rustDocs 'COPYRIGHT-library.html') -Destination $rustLicenses
Copy-Item -LiteralPath (Join-Path $rustDocs 'licenses') -Recurse -Destination $rustLicenses -Force
Write-Output "演示包已生成 $destination"
