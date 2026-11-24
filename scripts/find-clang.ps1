# Puts clang on PATH for the rest of a GitHub Actions job. ring needs it to
# build for ARM64 Windows. Uses a clang that is already installed when there
# is one: on PATH, standalone LLVM, or the one Visual Studio ships.
$ErrorActionPreference = "Stop"

function Use-Clang($dir) {
  Write-Host "Using clang from $dir"
  Add-Content -Path $env:GITHUB_PATH -Value $dir
  & (Join-Path $dir "clang.exe") --version
  exit 0
}

$onPath = Get-Command clang -ErrorAction SilentlyContinue
if ($onPath) {
  & $onPath.Source --version
  exit 0
}

$candidates = @("$env:ProgramFiles\LLVM\bin")
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
  foreach ($vs in & $vswhere -all -products * -property installationPath) {
    $candidates += "$vs\VC\Tools\Llvm\ARM64\bin", "$vs\VC\Tools\Llvm\x64\bin", "$vs\VC\Tools\Llvm\bin"
  }
}
foreach ($dir in $candidates) {
  if (Test-Path (Join-Path $dir "clang.exe")) { Use-Clang $dir }
}

Write-Host "No clang found. Installing LLVM."
choco install llvm -y --no-progress
if ($LASTEXITCODE -ne 0) { throw "Installing LLVM failed" }
Use-Clang "$env:ProgramFiles\LLVM\bin"
