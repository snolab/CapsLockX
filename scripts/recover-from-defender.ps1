# Recover CapsLockX after Windows Defender quarantines it.
#
# Defender's *behavioural* detection fires on the combination CLX legitimately
# needs: a global low-level keyboard hook, synthetic input injection, and — as
# of the plugin runner — starting other processes. That reads exactly like a
# keylogger to a heuristic, so the file is quarantined and the autostart task
# is torn down with it.
#
# Run it with no arguments; it re-launches itself elevated, so the only thing
# to do is accept the UAC prompt.
#
#   powershell -ExecutionPolicy Bypass -File scripts\recover-from-defender.ps1
#
# What it does, all of which needs administrator:
#   1. exclude the repo, so a restored binary is not immediately re-eaten
#   2. clear the recorded detections
#   3. put clx.exe back from the build output
#   4. recreate the CapsLockX_Rust logon task
#
# It is deliberately conservative: the exclusion covers this one folder, not a
# blanket rule, and nothing here disables Defender.

param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot),
    [switch]$Elevated
)

$ErrorActionPreference = 'Continue'

# ── self-elevate ────────────────────────────────────────────────────────────
$isAdmin = ([Security.Principal.WindowsPrincipal] `
    [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    if ($Elevated) {
        Write-Host "  elevation was requested but did not take effect." -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    Write-Host "  Asking for administrator — accept the UAC prompt." -ForegroundColor Cyan
    Write-Host ""
    $args = @(
        '-NoProfile', '-ExecutionPolicy', 'Bypass',
        '-File', "`"$PSCommandPath`"",
        '-Root', "`"$Root`"", '-Elevated'
    )
    try {
        $p = Start-Process powershell -Verb RunAs -ArgumentList $args -PassThru -Wait
        exit $p.ExitCode
    } catch {
        Write-Host "  UAC was declined — nothing changed." -ForegroundColor Yellow
        exit 1
    }
}

# Everything below is transcribed. The elevated window is its own console and
# closes when it finishes, so without this a failure is simply invisible —
# which is exactly what happened the first time this script ran.
$Log = Join-Path $env:TEMP "clx-recover-from-defender.log"
try { Stop-Transcript -ErrorAction SilentlyContinue | Out-Null } catch {}
Start-Transcript -Path $Log -Force | Out-Null

Write-Host ""
Write-Host "  CapsLockX — recovering from Defender" -ForegroundColor Cyan
Write-Host "  repo: $Root"
Write-Host "  log:  $Log"
Write-Host ""

# ── 1. exclusion ────────────────────────────────────────────────────────────
# First, or step 3 restores a file that is removed again moments later.
try {
    $existing = (Get-MpPreference).ExclusionPath
    if ($existing -contains $Root) {
        Write-Host "  [1/4] already excluded: $Root" -ForegroundColor DarkGray
    } else {
        Add-MpPreference -ExclusionPath $Root -ErrorAction Stop
        Write-Host "  [1/4] excluded from scanning: $Root" -ForegroundColor Green
    }
} catch {
    Write-Host "  [1/4] could not add the exclusion: $($_.Exception.Message)" -ForegroundColor Red
}

# ── 2. clear detections ─────────────────────────────────────────────────────
try {
    Remove-MpThreat -ErrorAction Stop
    Write-Host "  [2/4] cleared recorded detections" -ForegroundColor Green
} catch {
    Write-Host "  [2/4] nothing to clear, or already clean" -ForegroundColor DarkGray
}

# ── 3. restore the binary ───────────────────────────────────────────────────
$built = Join-Path $Root "rs\target\release\clx.exe"
$target = Join-Path $Root "clx.exe"
if (Test-Path $target) {
    Write-Host "  [3/4] clx.exe is already in place" -ForegroundColor DarkGray
} elseif (Test-Path $built) {
    try {
        Copy-Item $built $target -Force -ErrorAction Stop
        Write-Host "  [3/4] restored clx.exe from the build output" -ForegroundColor Green
    } catch {
        Write-Host "  [3/4] copy failed: $($_.Exception.Message)" -ForegroundColor Red
    }
} else {
    # Last resort: the backup this session took before touching anything.
    $backup = Get-ChildItem "$env:USERPROFILE\clx-backup-*.bin" -EA SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($backup) {
        Copy-Item $backup.FullName $target -Force
        Write-Host "  [3/4] restored clx.exe from $($backup.Name)" -ForegroundColor Green
    } else {
        Write-Host "  [3/4] no binary to restore — rebuild with .\build.ps1" -ForegroundColor Yellow
    }
}

# ── 4. autostart task ───────────────────────────────────────────────────────
# Elevated at logon, pointing at the repo-root exe, because that is where the
# self-updater installs. Matches what was there before Defender removed it.
if (Test-Path $target) {
    $null = schtasks /query /tn "CapsLockX_Rust" 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [4/4] CapsLockX_Rust already exists" -ForegroundColor DarkGray
    } else {
        # The full DOMAIN\user identity, not the bare name: Task Scheduler
        # rejects an unqualified user and the failure is easy to miss.
        $who = [Security.Principal.WindowsIdentity]::GetCurrent().Name
        $action  = New-ScheduledTaskAction -Execute $target -WorkingDirectory $Root
        $trigger = New-ScheduledTaskTrigger -AtLogOn -User $who
        # `Interactive`, not `InteractiveToken` — the latter is the COM API's
        # name for it and the cmdlet rejects it.
        $princ   = New-ScheduledTaskPrincipal -UserId $who `
                     -LogonType Interactive -RunLevel Highest
        $set     = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries `
                     -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([TimeSpan]::Zero) `
                     -StartWhenAvailable
        try {
            Register-ScheduledTask -TaskName "CapsLockX_Rust" -Action $action `
                -Trigger $trigger -Principal $princ -Settings $set -Force -ErrorAction Stop | Out-Null
            Write-Host "  [4/4] recreated the CapsLockX_Rust logon task" -ForegroundColor Green
        } catch {
            Write-Host "  [4/4] could not create the task: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
} else {
    Write-Host "  [4/4] skipped — no clx.exe to point the task at" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  Done. Start it with:  $target" -ForegroundColor Cyan
Write-Host "  Full log: $Log" -ForegroundColor DarkGray
Write-Host ""
try { Stop-Transcript | Out-Null } catch {}
Start-Sleep -Seconds 8
