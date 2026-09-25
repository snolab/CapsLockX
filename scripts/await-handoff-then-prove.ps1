# Wait for the keyboard hand-off, then run the watchdog proof.
#
# The agent cannot ask the user to press Start and watch for it in the same turn,
# so this waits on their behalf: it polls the hand-off state and runs the proof the
# moment the keyboard is offered, then writes the transcript where the agent can
# read it.
param([int]$WaitSeconds = 600)

$out = "$env:TEMP\clx-watchdog-proof.txt"
$repo = Split-Path -Parent $PSScriptRoot
Remove-Item $out -EA SilentlyContinue

function Say($msg) { Add-Content $out $msg }

Say "waiting for the keyboard hand-off (up to $WaitSeconds s)..."
$deadline = (Get-Date).AddSeconds($WaitSeconds)
$ready = $false
while ((Get-Date) -lt $deadline) {
    try {
        if ((Invoke-RestMethod "http://localhost:4550/api/handoff/current" -TimeoutSec 3).state -eq 'running') {
            $ready = $true
            break
        }
    } catch { }
    Start-Sleep -Milliseconds 500
}

if (-not $ready) { Say "TIMED OUT - the keyboard was never handed over."; exit 1 }

Say "keyboard handed over; running the smoke suite (it includes the watchdog proof)"
Say "----------------------------------------"
& powershell -NoProfile -ExecutionPolicy Bypass -File "$repo\scripts\clx-smoke.ps1" *>> $out
Say "----------------------------------------"
Say "exit code: $LASTEXITCODE"

# Always give the keyboard back, however the proof ended.
try {
    Invoke-RestMethod "http://localhost:4550/api/handoff/release" -Method Post `
        -ContentType application/json -Body '{"note":"proof finished"}' | Out-Null
    Say "keyboard released"
} catch { Say "could not release the hand-off: $($_.Exception.Message)" }
