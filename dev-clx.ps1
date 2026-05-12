# HMR-like dev loop for Windows: watch source, rebuild, copy, restart.
# Windows analogue of dev-clx.sh.
$ErrorActionPreference = "Stop"
$ROOT = $PSScriptRoot

if (-not (Get-Command cargo-watch -ErrorAction SilentlyContinue)) {
    Write-Host "[dev] installing cargo-watch (one-time)..."
    cargo install cargo-watch
    if ($LASTEXITCODE -ne 0) { throw "cargo install cargo-watch failed" }
}

Push-Location (Join-Path $ROOT "rs")
try {
    cargo watch `
        -w core/src `
        -w adapters/windows/src `
        -s "powershell -NoProfile -ExecutionPolicy Bypass -File `"$ROOT\build.ps1`""
} finally {
    Pop-Location
}
