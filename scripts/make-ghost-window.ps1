# Make a deliberately unresponsive window, so Windows raises a DWM "Ghost" in
# front of it — the exact pair CLX's window filter has to ignore.
#
# A window becomes "not responding" when its thread stops pumping messages for
# about five seconds. This creates a normal titled window, pumps long enough for
# the shell to notice it, then stops pumping on purpose.
#
#   .\clx-make-ghost.ps1              # hangs for 90s, then exits cleanly
#   .\clx-make-ghost.ps1 -Seconds 300
#
# Close it from Task Manager, or just wait — it exits on its own. Nothing here
# touches CLX; it only gives you something to cycle past.

param([int]$Seconds = 90)
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Windows.Forms, System.Drawing

$form                 = New-Object System.Windows.Forms.Form
$form.Text            = "CLX GHOST TEST - about to stop responding"
$form.Size            = New-Object System.Drawing.Size(560, 260)
$form.StartPosition   = "CenterScreen"
$form.TopMost         = $true

$label                = New-Object System.Windows.Forms.Label
$label.Dock           = "Fill"
$label.TextAlign      = "MiddleCenter"
$label.Font           = New-Object System.Drawing.Font("Segoe UI", 12)
$label.Text           = "Pumping messages for 3 seconds so the shell registers me...`n`nThen I stop responding for $Seconds seconds."
$form.Controls.Add($label)

$form.Show()
$form.Refresh()

# Pump briefly so the shell, the taskbar and CLX all see a healthy window first.
$until = (Get-Date).AddSeconds(3)
while ((Get-Date) -lt $until) { [System.Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 30 }

$label.Text = "NOT RESPONDING for $Seconds seconds.`n`nWindows will raise a Ghost window in front of me.`nCLX should ignore BOTH of us."
$form.Refresh()

# Windows only decides a window is "not responding" when something actually
# tries to talk to it and gets no answer. Left alone, a hung window just sits
# there looking healthy and no Ghost is ever raised — so arrange for someone to
# knock on the door, from a process that is still pumping. Written to a file
# rather than passed as -Command: the P/Invoke signature is full of quotes and
# nesting them inside an argument string is how you get a parser error.
$pokerPath = Join-Path $env:TEMP "clx-ghost-poker.ps1"
$pokerBody = @'
param([long]$Handle, [int]$Seconds)
Add-Type -Namespace K -Name P -MemberDefinition @"
[DllImport("user32.dll")]
public static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
"@
$h = [IntPtr]$Handle
$r = [IntPtr]::Zero
1..$Seconds | ForEach-Object {
  [void][K.P]::SendMessageTimeoutW($h, 0, [IntPtr]::Zero, [IntPtr]::Zero, 2, 300, [ref]$r)
  Start-Sleep -Seconds 1
}
'@
Set-Content -Path $pokerPath -Value $pokerBody -Encoding UTF8
$poker = Start-Process powershell -PassThru -WindowStyle Hidden -ArgumentList @(
  '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $pokerPath,
  '-Handle', ([int64]$form.Handle), '-Seconds', $Seconds
)

Write-Host ""
Write-Host "  Window is now hung for $Seconds seconds." -ForegroundColor Yellow
Write-Host "  Give Windows ~5s to raise the Ghost, then try CLX+Z." -ForegroundColor Yellow
Write-Host "  Expected: cycling skips both the hung window and its ghost." -ForegroundColor Cyan
Write-Host ""

# Stop pumping. This is the whole point: the thread is alive, the window is not.
#
# NOT `Start-Sleep`: PowerShell runs STA for WinForms, and an STA thread keeps
# pumping messages during its blocking waits, so the window stays responsive and
# Windows never raises a Ghost. Thread.Sleep blocks the thread outright.
[System.Threading.Thread]::Sleep($Seconds * 1000)

if ($poker -and -not $poker.HasExited) { $poker | Stop-Process -Force -ErrorAction SilentlyContinue }
$form.Close()
$form.Dispose()
Write-Host "  ghost test window closed" -ForegroundColor Green
