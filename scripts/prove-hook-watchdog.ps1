# Prove that clx notices a dropped keyboard hook and replaces it.
#
# The watchdog had already failed silently in the field — it was sitting on the UI
# thread's timer, which is the thread that dies when a hook callback blocks. It
# now lives on the ticker, and a recovery path nobody has watched recover is a
# guess, so this watches it.
#
#   1. requires the keyboard hand-off page to be RUNNING, so the test keystrokes
#      land in the browser and not in whatever the user was typing
#   2. starts clx with CLX_TEST_DROP_HOOK_MS, which drops its own hook exactly as
#      Windows does when a callback overruns LowLevelHooksTimeout
#   3. feeds it inert Shift presses — never suppressed by clx, so they always
#      reach the raw-input witness the watchdog compares against
#   4. asserts the log shows detection, a new hook generation, and keys arriving
#      again afterwards
#
#   bun lab/serve.ts                       # then open /lab/keyboard-handoff/
#   powershell -ExecutionPolicy Bypass -File scripts\prove-hook-watchdog.ps1
#
# Run it on a machine with no orphaned hooks from earlier wedged instances: a
# stale hook from a process that will not die changes who sees what, and every
# result becomes untrustworthy. A reboot is the only way to clear those.

param(
    [int]$DropAfterMs = 15000,
    [switch]$DryRun          # check prerequisites and exit
)

$ErrorActionPreference = 'Continue'
$repo = Split-Path -Parent $PSScriptRoot
$log = "$env:TEMP\capslockx_hook.log"
$handoff = "http://localhost:4550/api/handoff"

function Fail($msg) { Write-Host "  FAIL  $msg" -ForegroundColor Red; exit 1 }
function Ok($msg)   { Write-Host "  ok    $msg" -ForegroundColor Green }
function Info($msg) { Write-Host "        $msg" -ForegroundColor DarkGray }

Write-Host ""
Write-Host "  clx hook watchdog proof" -ForegroundColor Cyan
Write-Host ""

# ── 1. prerequisites ────────────────────────────────────────────────────────
$zombies = Get-Process clx -EA SilentlyContinue | Where-Object { -not $_.Responding }
if ($zombies) {
    Fail ("wedged clx instance(s) present (" + ($zombies.Id -join ', ') + "). Their orphaned hooks would corrupt the result; reboot first.")
}

try { $state = (Invoke-RestMethod "$handoff/current" -TimeoutSec 5).state }
catch { Fail "lab server not reachable. Run: bun lab/serve.ts" }

if ($state -ne 'running') {
    Fail "the keyboard is not handed over (state=$state). Open http://localhost:4550/lab/keyboard-handoff/ and press Start the test."
}
Ok "keyboard handed over"

if ($DryRun) { Info "dry run - prerequisites only"; exit 0 }

# ── 2. start clx, primed to drop its own hook ───────────────────────────────
& "$repo\clx.exe" quit | Out-Null
Start-Sleep 2
Remove-Item $log -EA SilentlyContinue
$env:CLX_DEBUG = '1'
$env:CLX_TEST_DROP_HOOK_MS = "$DropAfterMs"
New-Item -ItemType Directory -Force "$repo\tmp" | Out-Null
Start-Process -FilePath "$repo\clx.exe" -WorkingDirectory $repo `
    -RedirectStandardError "$repo\tmp\clx-stderr.log" `
    -RedirectStandardOutput "$repo\tmp\clx-stdout.log"

function WaitForLog([string]$pattern, [int]$seconds, [string]$what) {
    for ($i = 0; $i -lt $seconds * 4; $i++) {
        if ((Get-Content $log -EA SilentlyContinue | Select-String $pattern)) { Ok $what; return $true }
        Start-Sleep -Milliseconds 250
    }
    Write-Host "  FAIL  timed out waiting for: $what" -ForegroundColor Red
    return $false
}

if (-not (WaitForLog "gen 1 installed and pumping" 40 "hook generation 1 installed")) { exit 1 }

# ── 3. let it drop the hook, then feed the witness ──────────────────────────
if (-not (WaitForLog "TEST: dropping the hook on purpose" ([int]($DropAfterMs / 1000) + 15) "hook dropped on purpose")) { exit 1 }

Add-Type -TypeDefinition 'using System;using System.Runtime.InteropServices;public class PW{[DllImport("user32.dll")]public static extern void keybd_event(byte v,byte s,uint f,UIntPtr e);}'
Info "feeding inert Shift presses (never suppressed, so raw input always sees them)"
for ($i = 0; $i -lt 6; $i++) {
    [PW]::keybd_event(0xA0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 90
    [PW]::keybd_event(0xA0, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 250
}

# ── 4. did it notice, and did it come back? ─────────────────────────────────
if (-not (WaitForLog "gone or wedged" 20 "watchdog detected the dead hook")) { exit 1 }
if (-not (WaitForLog "gen 2 installed and pumping" 20 "replacement hook installed")) { exit 1 }

# The real question: are keystrokes reaching clx again?
$before = (Get-Content $log | Select-String "\[hook\] vk=0xA0").Count
[PW]::keybd_event(0xA0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 90
[PW]::keybd_event(0xA0, 0, 2, [UIntPtr]::Zero)
Start-Sleep 2
$after = (Get-Content $log | Select-String "\[hook\] vk=0xA0").Count
if ($after -gt $before) { Ok "keys reach the replacement hook ($before -> $after)" }
else { Fail "the replacement hook is not receiving keys" }

Invoke-RestMethod "$handoff/release" -Method Post -ContentType application/json `
    -Body '{"note":"hook watchdog proof finished"}' | Out-Null

Write-Host ""
Write-Host "  PASSED - the watchdog detected a dropped hook and recovered from it." -ForegroundColor Green
Write-Host "  The keyboard is yours again." -ForegroundColor Cyan
Write-Host ""
