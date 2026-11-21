param([string]$SevenZip = 'C:\Program Files\7-Zip\7z.exe')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$toolsRoot = Join-Path $projectRoot '.tools'
$archive = Join-Path $toolsRoot 'LLVM-21.1.3-win64.exe'
$destination = Join-Path $toolsRoot 'llvm-21.1.3'
$expectedHash = '5d2de254ff5baad330db188438362bfeab4e98c6cc872d55f4b81ee82403164b'
New-Item -ItemType Directory -Path $toolsRoot -Force | Out-Null
if (-not (Test-Path -LiteralPath $archive)) {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri 'https://github.com/llvm/llvm-project/releases/download/llvmorg-21.1.3/LLVM-21.1.3-win64.exe' -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expectedHash) {
    throw 'LLVM archive hash mismatch'
}
if (-not (Test-Path -LiteralPath $SevenZip)) { throw 'Provide -SevenZip with the path to 7z.exe' }
& $SevenZip x $archive "-o$destination" '-y' 'bin/LLVM-C.dll'
if ($LASTEXITCODE -ne 0) { throw 'LLVM extraction failed' }
$licensePath = Join-Path $destination 'LLVM-LICENSE.txt'
if (-not (Test-Path -LiteralPath $licensePath)) {
    Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-21.1.3/llvm/LICENSE.TXT' -OutFile $licensePath
}
Write-Output "LLVM DLL: $destination\bin\LLVM-C.dll"

