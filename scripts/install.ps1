# CapsLockX installer for Windows
# Usage: irm https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.ps1 | iex
$ErrorActionPreference = "Stop"

$Repo = "snolab/CapsLockX"
$InstallDir = "$env:LOCALAPPDATA\CapsLockX"
$BinName = "clx.exe"

# Find latest release
if ($env:CLX_VERSION) { $Tag = $env:CLX_VERSION }
else {
    $releases = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases?per_page=10"
    $Tag = $releases[0].tag_name
}
Write-Host "Installing CapsLockX $Tag for Windows..."

# Download and extract
$ZipUrl = "https://github.com/$Repo/releases/download/$Tag/CapsLockX-windows-x86_64.zip"
$TmpZip = Join-Path $env:TEMP "capslockx.zip"
Invoke-WebRequest -Uri $ZipUrl -OutFile $TmpZip

# Extract
if (Test-Path $InstallDir) { Remove-Item -Recurse -Force $InstallDir }
Expand-Archive -Path $TmpZip -DestinationPath $InstallDir -Force
Remove-Item $TmpZip

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
    Write-Host "Added $InstallDir to PATH (restart terminal to take effect)."
}

# Create Start Menu shortcut
$StartMenu = [Environment]::GetFolderPath("StartMenu")
$ShortcutPath = Join-Path $StartMenu "CapsLockX.lnk"
$WScriptShell = New-Object -ComObject WScript.Shell
$Shortcut = $WScriptShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = Join-Path $InstallDir $BinName
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "CapsLockX keyboard productivity tool"
$Shortcut.Save()

Write-Host "Installed to $InstallDir ($Tag)"
Write-Host "Run 'clx' to start CapsLockX."
