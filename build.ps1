# Build CapsLockX (Windows), copy binary to root, and auto-restart.
# Windows analogue of build.sh. Run from any directory.
$ErrorActionPreference = "Stop"
$ROOT = $PSScriptRoot

Push-Location (Join-Path $ROOT "rs")
try {
    # The Slint windows are libraries inside clx.exe now. They are still
    # built as standalone binaries so the tools stay runnable on their own,
    # but nothing is staged next to clx.exe any more.
    cargo build -p capslockx-windows -p clx-prefs-slint -p clx-prompt-slint --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
} finally {
    Pop-Location
}

$CARGO_BIN = Join-Path $ROOT "rs\target\release\clx.exe"
$CLX_BIN   = Join-Path $ROOT "clx.exe"

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

# Running clx.exe holds a file lock on the root copy, so it has to go before we
# can overwrite it. Ask it to quit rather than killing it: a killed clx never
# uninstalls its keyboard hook, and since its threads sit in win32k it then
# refuses to finish dying — leaving an orphaned hook that swallows every Space
# press. Recovering from that needs a reboot, so this is worth the extra second.
if (Test-Path $CLX_BIN) { & $CLX_BIN quit | Out-Null }
for ($i = 0; $i -lt 20 -and (Get-Process clx -ErrorAction SilentlyContinue); $i++) {
    Start-Sleep -Milliseconds 250
}
# Only if it ignored the request. A hook orphaned here is still better than a
# build that cannot deploy, but it should be rare enough to notice.
$stubborn = Get-Process clx -ErrorAction SilentlyContinue
if ($stubborn) {
    Write-Host "[build] clx did not quit when asked - falling back to killing it"
    $stubborn | ForEach-Object { try { $_.Kill() } catch { } }
    Start-Sleep -Milliseconds 500
}

# A clx that wedged can leave a process Windows will not finish killing, and it
# keeps the lock on its own image for as long as it exists. Copying then fails,
# and with ErrorActionPreference=Stop the whole build aborted *before* the copy —
# so the build "succeeded" while quietly deploying nothing, which cost an hour of
# debugging a fix that was never actually running.
#
# Windows lets you rename a locked image even though it will not let you replace
# one. So move the old file aside and copy into the freed name. The stale
# `clx.exe.old-*` files are cleaned up at the next startup by
# self_update::cleanup_old_binaries.
try {
    Copy-Item $CARGO_BIN $CLX_BIN -Force -ErrorAction Stop
} catch {
    $aside = "$CLX_BIN.old-" + (Get-Date -Format "yyyyMMddHHmmss")
    Write-Host "[build] clx.exe is locked (wedged instance?) - renaming it aside"
    Move-Item $CLX_BIN $aside -Force
    Copy-Item $CARGO_BIN $CLX_BIN -Force
}

# Prove the deploy landed rather than trusting it: the hashes must now agree.
$deployed = (Get-FileHash $CLX_BIN -Algorithm SHA256).Hash
if ($deployed -ne $cargoHash) { throw "deploy failed: clx.exe does not match the build output" }

Start-Process -FilePath $CLX_BIN -WorkingDirectory $ROOT
Write-Host "[build] clx restarted (new binary)"
