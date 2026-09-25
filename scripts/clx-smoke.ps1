# The checks that kept getting done by hand, and one that was skipped and cost a
# working keyboard.
#
# Every failure this suite covers actually happened on a live machine in one
# afternoon: injection silently dropped so the space bar died, an instance
# force-killed so its hook was orphaned and only a reboot brought Space back, and
# a hook watchdog that could never fire because it sat on the thread that dies.
# Each was found by reading the hook log by hand, twice, because nothing checked.
#
#   bun lab/serve.ts      # then open /lab/keyboard-handoff/ and press Start
#   powershell -ExecutionPolicy Bypass -File scripts\clx-smoke.ps1
#
# The hook log is the oracle throughout: what clx believed it saw and did, rather
# than what a window happened to display. Tests that press keys need the keyboard
# handed over, because injected keys land in whatever has focus — on a machine
# someone is using, that is their editor.

param(
    [switch]$NoKeys,        # skip everything that injects keystrokes
    [switch]$SkipWatchdog   # skip the slow hook-recovery proof
)

$ErrorActionPreference = 'Continue'
$repo = Split-Path -Parent $PSScriptRoot
$log  = "$env:TEMP\capslockx_hook.log"
$clx  = Join-Path $repo "clx.exe"
$script:failures = 0

function Pass($m) { Write-Host "  PASS  $m" -ForegroundColor Green }
function Fail($m) { Write-Host "  FAIL  $m" -ForegroundColor Red; $script:failures++ }
function Note($m) { Write-Host "        $m" -ForegroundColor DarkGray }
function Head($m) { Write-Host ""; Write-Host "  $m" -ForegroundColor Cyan }

Add-Type -TypeDefinition 'using System;using System.Runtime.InteropServices;public class SmokeKey{[DllImport("user32.dll")]public static extern void keybd_event(byte v,byte s,uint f,UIntPtr e);}'

function Tap([byte]$vk, [int]$holdMs = 70) {
    [SmokeKey]::keybd_event($vk, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds $holdMs
    [SmokeKey]::keybd_event($vk, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 400
}

function LogLines { Get-Content $log -EA SilentlyContinue }
function LogCount { (LogLines).Count }
function Since([int]$n) { LogLines | Select-Object -Skip $n }

function StartClx([hashtable]$extraEnv = @{}, [switch]$KeepLog) {
    # Note the log cannot be deleted while clx is running: its writer holds the
    # file open without FILE_SHARE_DELETE, so Remove-Item fails silently and the
    # log stays cumulative across instances. Checks therefore identify instances
    # by `self_pid=` rather than by "everything after line N", which is what made
    # test 4 quietly measure nothing.
    if (-not $KeepLog) { Remove-Item $log -EA SilentlyContinue }
    $env:CLX_DEBUG = '1'
    foreach ($k in $extraEnv.Keys) { Set-Item "env:$k" $extraEnv[$k] }
    New-Item -ItemType Directory -Force (Join-Path $repo "tmp") | Out-Null
    $p = Start-Process -FilePath $clx -WorkingDirectory $repo -PassThru `
        -RedirectStandardError (Join-Path $repo "tmp\smoke-err.log") `
        -RedirectStandardOutput (Join-Path $repo "tmp\smoke-out.log")
    foreach ($k in $extraEnv.Keys) { Remove-Item "env:$k" -EA SilentlyContinue }
    $p
}

function WaitLog([string]$pattern, [int]$seconds) {
    for ($i = 0; $i -lt $seconds * 4; $i++) {
        if (LogLines | Select-String $pattern) { return $true }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

Write-Host ""
Write-Host "  clx smoke test" -ForegroundColor Cyan

# ── keyboard hand-off, for the tests that press keys ────────────────────────
$haveKeyboard = $false
if (-not $NoKeys) {
    try {
        $haveKeyboard = (Invoke-RestMethod "http://localhost:4550/api/handoff/current" -TimeoutSec 3).state -eq 'running'
    } catch { }
    if (-not $haveKeyboard) {
        Write-Host ""
        Fail "keyboard not handed over - open http://localhost:4550/lab/keyboard-handoff/ and press Start (or pass -NoKeys)"
        Note "injected keys would otherwise land in whatever window has focus"
        exit 1
    }
    Note "keyboard handed over"
}

# ── 1. it starts, and the hook gets its own thread ──────────────────────────
Head "1. startup"
& $clx quit | Out-Null
Start-Sleep 2
$null = StartClx
if (WaitLog "installed and pumping" 45) {
    Pass "hook installed on its own thread"
} else {
    Fail "hook never reported 'installed and pumping'"
    exit 1
}
$deaf = LogLines | Select-String "\[main\] started|\[main\] hook installed"
if ($deaf.Count -ge 2) {
    $t0 = [long]($deaf[0].Line -split ' ')[0]
    $t1 = [long]($deaf[-1].Line -split ' ')[0]
    if (($t1 - $t0) -lt 2000) { Pass "deaf window at startup: $($t1 - $t0) ms" }
    else { Fail "deaf for $($t1 - $t0) ms before the hook was installed (was 10353 ms once)" }
}

if ($haveKeyboard) {
    # ── 2. the hook actually receives keys ──────────────────────────────────
    Head "2. the hook receives keys"
    $n = LogCount
    Tap 0xA0     # lone Shift: traverses the chain, types nothing
    if (Since $n | Select-String "vk=0xA0") { Pass "keystrokes reach the hook" }
    else { Fail "the hook received nothing (another hook swallowing, or ours is dead)" }

    # ── 3. a Space tap still produces a space ───────────────────────────────
    # The regression that killed the space bar: clx suppresses both halves of a
    # Space press and injects a replacement. No replacement, no space, and the
    # only symptom is a keyboard that cannot type spaces.
    Head "3. Space tap injects its replacement"
    $n = LogCount
    Tap 0x20
    $ours = Since $n | Select-String "vk=0x20.*ours=true"
    if ($ours) { Pass "replacement Space injected ($($ours.Count) event(s))" }
    else { Fail "Space was suppressed with no replacement - the space bar is dead" }
}

# ── 4. replacing an instance does not orphan its hook ───────────────────────
# This is what stranded seven hooks in one afternoon. An instance force-killed
# never uninstalls its hook, and its corpse will not finish dying, so the orphan
# keeps suppressing Space for ever.
Head "4. instance replacement is graceful"
$old = (Get-Process clx -EA SilentlyContinue | Where-Object { $_.Responding } | Select-Object -First 1).Id
if (-not $old) {
    Fail "no live instance to replace - test 1 should have left one running"
} else {
    $b = StartClx -KeepLog
    Start-Sleep 18
    # Identified by pid, not by log position: the log is cumulative and shared.
    #
    # The assertion that matters is "not force-killed", not "logged as graceful".
    # Since the quit watcher leaves via hard_exit the victim is often gone before
    # the replacing instance even opens a handle to it, which is a better outcome
    # than the graceful-wait path and logs differently.
    if (LogLines | Select-String "force-killing previous pid=$old") {
        Fail "pid $old was force-killed by pid $($b.Id) - its hook is now orphaned"
    } elseif (Get-Process -Id $old -EA SilentlyContinue) {
        Fail "pid $old is still present after being replaced by pid $($b.Id)"
    } else {
        Pass "pid $old went away without being force-killed (replaced by $($b.Id))"
    }
    if (LogLines | Select-String "needs elevation") {
        Fail "a fast graceful exit was misread as 'needs elevation' - spurious elevated relaunch"
    }
}

# ── 5. quit leaves nothing behind ───────────────────────────────────────────
Head "5. clx quit"
$live = (Get-Process clx -EA SilentlyContinue | Select-Object -First 1).Id
& $clx quit | Out-Null
for ($i = 0; $i -lt 24 -and (Get-Process -Id $live -EA SilentlyContinue); $i++) { Start-Sleep -Milliseconds 250 }
$corpse = Get-Process -Id $live -EA SilentlyContinue
if ($corpse) { Fail "pid $live survived the quit ($($corpse.Threads.Count) thread(s)) - hook orphaned" }
else { Pass "pid $live exited completely" }

# ── 6. no undead instances anywhere ─────────────────────────────────────────
Head "6. no undead instances"
$undead = Get-Process clx -EA SilentlyContinue | Where-Object { -not $_.Responding }
if ($undead) {
    Fail "undead clx present: $($undead.Id -join ', ') - these hold hooks; run: clx unstick"
} else {
    Pass "none"
}

# ── 7. the watchdog recovers a dropped hook ─────────────────────────────────
if (-not $SkipWatchdog -and $haveKeyboard) {
    Head "7. hook watchdog recovery"
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "prove-hook-watchdog.ps1")
    if ($LASTEXITCODE -eq 0) { Pass "watchdog detected a dropped hook and replaced it" }
    else { Fail "watchdog proof failed (see its output above)" }
} elseif (-not $SkipWatchdog) {
    Note "7. watchdog proof skipped - needs the keyboard"
}

Write-Host ""
if ($script:failures -eq 0) {
    Write-Host "  ALL PASSED" -ForegroundColor Green
} else {
    Write-Host "  $($script:failures) FAILED" -ForegroundColor Red
}
Write-Host ""
exit $script:failures
