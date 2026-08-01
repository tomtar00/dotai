# Install dotai onto your PATH (Windows).
#
# Usage:
#   .\scripts\install.ps1
# Custom directory (default ~\.local\bin):
#   $env:DOTAI_INSTALL_DIR = "C:\Tools"; .\scripts\install.ps1
#
# Tip: `cargo install --path .` is simpler — it installs to ~\.cargo\bin,
# which rustup already puts on your PATH.
# For a custom location: `cargo install --path . --root C:\Tools` (-> C:\Tools\bin).
#
# If execution policy blocks scripts, run:
#   powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$destDir = if ($env:DOTAI_INSTALL_DIR) { $env:DOTAI_INSTALL_DIR } else { "$HOME\.local\bin" }

Push-Location $root
try { cargo build --release } finally { Pop-Location }

New-Item -ItemType Directory -Force -Path $destDir | Out-Null
Copy-Item -Force (Join-Path $root "target\release\dotai.exe") (Join-Path $destDir "dotai.exe")
Write-Host "Installed dotai to $destDir\dotai.exe"

if (@($env:PATH -split ";" | Where-Object { $_.TrimEnd("\") -ieq $destDir }).Count -gt 0) {
    Write-Host "dotai is on your PATH. Run 'dotai --help'."
} else {
    Write-Host "Add $destDir to your PATH: `$env:PATH = `"$destDir;`$env:PATH`""
}
