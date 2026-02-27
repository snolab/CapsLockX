#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Creates a Hyper-V Ubuntu 24.04 VM with Rust + evdev tools pre-installed
    for testing the CapsLockX Linux adapter.

.DESCRIPTION
    Downloads the Ubuntu cloud image (VHDX), generates a cloud-init seed ISO,
    and creates a Gen-2 Hyper-V VM.  On first boot cloud-init installs Rust,
    evdev libs, and clones the CapsLockX repo.

.NOTES
    Run from an elevated PowerShell prompt:
        powershell -ExecutionPolicy Bypass -File scripts\setup-linux-vm.ps1
#>
param(
    [string]$VMName      = "clx-linux-dev",
    [int]   $MemoryMB    = 4096,
    [int]   $CPUs        = 4,
    [int]   $DiskSizeGB  = 30,
    [string]$RepoURL     = "https://github.com/snolab/CapsLockX.git",
    [string]$RepoBranch  = "main"
)

$ErrorActionPreference = "Stop"
$VMDir     = "$env:USERPROFILE\.hyperv\$VMName"
$VHDXPath  = "$VMDir\disk.vhdx"
$SeedISO   = "$VMDir\seed.iso"
$CloudURL  = "https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.vhdx"

# ═══════════════════════════════════════════════════════════════════════════════
# Helper: create ISO from a directory using IMAPI2FS COM + C# IStream reader
# ═══════════════════════════════════════════════════════════════════════════════

Add-Type -TypeDefinition @"
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;

public static class ISOHelper {
    public static void WriteIStreamToFile(object comStream, string path) {
        IStream s = (IStream)comStream;
        STATSTG stat;
        s.Stat(out stat, 0);
        long remaining = stat.cbSize;

        using (var fs = new FileStream(path, FileMode.Create, FileAccess.Write)) {
            byte[] buf = new byte[65536];
            while (remaining > 0) {
                int want  = (int)Math.Min(buf.Length, remaining);
                IntPtr pRead = Marshal.AllocCoTaskMem(sizeof(int));
                try {
                    s.Read(buf, want, pRead);
                    int got = Marshal.ReadInt32(pRead);
                    if (got == 0) break;
                    fs.Write(buf, 0, got);
                    remaining -= got;
                } finally {
                    Marshal.FreeCoTaskMem(pRead);
                }
            }
        }
    }
}
"@

function New-CloudInitISO([string]$SourceDir, [string]$OutputPath) {
    $fsi = New-Object -ComObject IMAPI2FS.MsftFileSystemImage
    $fsi.FileSystemsToCreate = 3          # ISO 9660 + Joliet
    $fsi.VolumeName          = "cidata"   # cloud-init NoCloud label

    foreach ($f in Get-ChildItem $SourceDir -File) {
        $stream = New-Object -ComObject ADODB.Stream
        $stream.Open()
        $stream.Type = 1   # binary
        $stream.LoadFromFile($f.FullName)
        $fsi.Root.AddStream($f.Name, $stream)
    }

    $result = $fsi.CreateResultImage()
    [ISOHelper]::WriteIStreamToFile($result.ImageStream, $OutputPath)
    Write-Host "  Seed ISO: $OutputPath" -ForegroundColor Green
}

# ═══════════════════════════════════════════════════════════════════════════════
Write-Host ""
Write-Host "  CapsLockX Linux Dev VM Setup" -ForegroundColor Cyan
Write-Host "  =============================" -ForegroundColor Cyan
Write-Host ""

# ── 1. Working directory ─────────────────────────────────────────────────────
New-Item -ItemType Directory -Path $VMDir -Force | Out-Null

# ── 2. Download Ubuntu cloud image ──────────────────────────────────────────
$BaseVHDX = "$VMDir\ubuntu-cloud-base.vhdx"
if (!(Test-Path $BaseVHDX)) {
    Write-Host "[1/6] Downloading Ubuntu 24.04 cloud image (~700 MB)..." -ForegroundColor Yellow
    $ProgressPreference = 'SilentlyContinue'   # speed up Invoke-WebRequest
    Invoke-WebRequest -Uri $CloudURL -OutFile $BaseVHDX -UseBasicParsing
    $ProgressPreference = 'Continue'
    Write-Host "  Downloaded." -ForegroundColor Green
} else {
    Write-Host "[1/6] Cloud image already cached." -ForegroundColor Green
}

# ── 3. Copy & resize VHDX ───────────────────────────────────────────────────
if (!(Test-Path $VHDXPath)) {
    Write-Host "[2/6] Copying and resizing VHDX to ${DiskSizeGB} GB..." -ForegroundColor Yellow
    Copy-Item $BaseVHDX $VHDXPath
    Resize-VHD -Path $VHDXPath -SizeBytes ($DiskSizeGB * 1GB)
    Write-Host "  Done." -ForegroundColor Green
} else {
    Write-Host "[2/6] VHDX already exists." -ForegroundColor Green
}

# ── 4. Cloud-init files ─────────────────────────────────────────────────────
Write-Host "[3/6] Generating cloud-init seed..." -ForegroundColor Yellow
$CIDir = "$VMDir\cidata"
New-Item -ItemType Directory -Path $CIDir -Force | Out-Null

# meta-data
Set-Content "$CIDir\meta-data" -NoNewline -Encoding UTF8 @"
instance-id: clx-dev-001
local-hostname: clx-dev
"@

# user-data
Set-Content "$CIDir\user-data" -NoNewline -Encoding UTF8 @"
#cloud-config
hostname: clx-dev
manage_etc_hosts: true

users:
  - name: dev
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    lock_passwd: false
    plain_text_passwd: dev
    groups: [input, sudo]

package_update: true
packages:
  - build-essential
  - pkg-config
  - libevdev-dev
  - git
  - curl
  - vim
  - htop
  - usbutils

runcmd:
  # Install Rust
  - su - dev -c 'curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
  # Clone repo
  - su - dev -c 'git clone --branch $RepoBranch $RepoURL ~/CapsLockX || true'
  # Build the Linux adapter
  - su - dev -c '. ~/.cargo/env && cd ~/CapsLockX/rs && cargo build -p capslockx-linux --release 2>&1 | tail -5'
  # Helpful MOTD
  - |
    cat > /etc/motd << 'MOTD'

    ============================================
      CapsLockX Linux Dev VM
    ============================================
      User: dev / dev
      Repo: ~/CapsLockX
      Build: cd ~/CapsLockX/rs && cargo build -p capslockx-linux --release
      Run:   sudo ./target/release/capslockx-linux
      Hint:  evdev needs root or "input" group.
    ============================================

    MOTD

final_message: "Cloud-init finished in \$UPTIME seconds."
"@

# ── 5. Create seed ISO ──────────────────────────────────────────────────────
Write-Host "[4/6] Creating seed ISO..." -ForegroundColor Yellow
New-CloudInitISO -SourceDir $CIDir -OutputPath $SeedISO

# ── 6. Create Hyper-V VM ────────────────────────────────────────────────────
$existing = Get-VM -Name $VMName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "[5/6] VM '$VMName' already exists – recreating..." -ForegroundColor Yellow
    Stop-VM   -Name $VMName -Force -TurnOff -ErrorAction SilentlyContinue
    Remove-VM -Name $VMName -Force
} else {
    Write-Host "[5/6] Creating Hyper-V VM..." -ForegroundColor Yellow
}

New-VM -Name $VMName `
       -MemoryStartupBytes ($MemoryMB * 1MB) `
       -VHDPath $VHDXPath `
       -Generation 2 `
       -SwitchName "Default Switch" | Out-Null

Set-VM -Name $VMName `
       -ProcessorCount $CPUs `
       -AutomaticCheckpointsEnabled $false

# Gen-2 needs Secure Boot off for Ubuntu cloud images
Set-VMFirmware -VMName $VMName -EnableSecureBoot Off

# Attach seed ISO as DVD
Add-VMDvdDrive -VMName $VMName -Path $SeedISO

Write-Host "  VM created." -ForegroundColor Green

# ── 7. Start ─────────────────────────────────────────────────────────────────
Write-Host "[6/6] Starting VM..." -ForegroundColor Yellow
Start-VM -Name $VMName

Write-Host ""
Write-Host "  ======================================" -ForegroundColor Cyan
Write-Host "  VM '$VMName' is booting!" -ForegroundColor Cyan
Write-Host "  ======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Credentials : dev / dev"
Write-Host "  Console     : vmconnect localhost $VMName"
Write-Host "  Find IP     : (Get-VM $VMName | Get-VMNetworkAdapter).IPAddresses"
Write-Host "  SSH          : ssh dev@<ip>"
Write-Host ""
Write-Host "  Cloud-init installs Rust + tools on first boot (~3-5 min)."
Write-Host "  Check progress:  sudo cloud-init status --wait"
Write-Host ""
