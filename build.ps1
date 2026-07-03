# Build CapsLockX (Windows), copy binary to root, and auto-restart.
# Windows analogue of build.sh. Run from any directory.
$ErrorActionPreference = "Stop"
$ROOT = $PSScriptRoot

Push-Location (Join-Path $ROOT "rs")
try {
    cargo build -p capslockx-windows -p clx-prefs-slint --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
} finally {
    Pop-Location
}

$CARGO_BIN = Join-Path $ROOT "rs\target\release\clx.exe"
$CLX_BIN   = Join-Path $ROOT "clx.exe"

# Keep the native prefs binary next to clx.exe so open_prefs_window finds it.
$PREFS_SRC = Join-Path $ROOT "rs\target\release\clx-prefs-slint.exe"
$PREFS_DST = Join-Path $ROOT "clx-prefs-slint.exe"
if (Test-Path $PREFS_SRC) {
    Get-Process clx-prefs-slint -ErrorAction SilentlyContinue | Stop-Process -Force
    Copy-Item $PREFS_SRC $PREFS_DST -Force -ErrorAction SilentlyContinue
}

# Skip the copy + relaunch when the binary is unchanged. Avoids needlessly
# killing a running instance when only a doc or comment changed.
$cargoHash = (Get-FileHash $CARGO_BIN -Algorithm SHA256).Hash
$clxHash   = if (Test-Path $CLX_BIN) { (Get-FileHash $CLX_BIN -Algorithm SHA256).Hash } else { "" }

if ($cargoHash -eq $clxHash) {
    # Binary is identical to the deployed copy. Still launch if nothing is
    # running (e.g. first `dev` after clx was closed) so the loop reliably
    # starts clx; otherwise leave the running instance untouched.
    if (-not (Get-Process clx -ErrorAction SilentlyContinue)) {
        Start-Process -FilePath $CLX_BIN -WorkingDirectory $ROOT
        Write-Host "[build] binary unchanged - clx was not running, started it"
    } else {
        Write-Host "[build] done - binary unchanged"
    }
    exit 0
}

# Running clx.exe holds a file lock on the root copy; kill it before
# overwriting. The new instance will auto-deduplicate via shm.rs anyway.
Get-Process clx -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300

Copy-Item $CARGO_BIN $CLX_BIN -Force

Start-Process -FilePath $CLX_BIN -WorkingDirectory $ROOT
Write-Host "[build] clx restarted (new binary)"
