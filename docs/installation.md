# CapsLockX Installation Guide

## Quick Install (one-liner)

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.ps1 | iex
```

## Download from GitHub Releases

Latest release: <https://github.com/snolab/CapsLockX/releases>

### Windows

| File | Description |
|------|-------------|
| `CapsLockX-setup-x86_64.exe` | **Recommended.** NSIS installer — adds to PATH, Start Menu, and Add/Remove Programs. |
| `CapsLockX-windows-x86_64.zip` | Portable zip with `clx.exe`, `install.ps1`, and `uninstall.ps1`. |

**Setup.exe** installs to `%LOCALAPPDATA%\CapsLockX` and registers in Add/Remove Programs for clean uninstall.

**Portable zip**: extract anywhere and run `clx.exe`, or run `install.ps1` to install to the default location.

### macOS

| File | Description |
|------|-------------|
| `CapsLockX-macos-arm64.dmg` | **Recommended.** Drag-and-drop DMG containing `clx` binary and install script. |
| `CapsLockX-macos-arm64.tar.gz` | Tarball with `clx` binary and `install.sh`. |

After installing, grant **Accessibility** permission:
> System Settings → Privacy & Security → Accessibility → enable `clx`

The binary is ad-hoc code-signed with identifier `com.snomiao.capslockx` so Accessibility permission persists across updates.

### Linux

| File | Description |
|------|-------------|
| `CapsLockX-linux-x86_64.deb` | **Recommended for Debian/Ubuntu.** Installs `clx` to `/usr/local/bin/`. |
| `CapsLockX-linux-x86_64.tar.gz` | Tarball with `clx` binary and `install.sh`. |

```bash
# Debian/Ubuntu
sudo dpkg -i CapsLockX-linux-x86_64.deb

# Or manual install
tar xzf CapsLockX-linux-x86_64.tar.gz
sudo cp CapsLockX-*/clx /usr/local/bin/
```

Linux requires `evdev` access. Run as root or add your user to the `input` group:
```bash
sudo usermod -aG input $USER
# Log out and back in for the group change to take effect
```

## Install a specific version

```bash
# Unix
CLX_VERSION=v2.0.0-beta.1 curl -fsSL .../install.sh | bash

# Windows
$env:CLX_VERSION="v2.0.0-beta.1"; irm .../install.ps1 | iex
```

## Build from source

```bash
git clone https://github.com/snolab/CapsLockX.git
cd CapsLockX

# macOS
./build.sh

# Windows
cd rs && cargo build -p capslockx-windows --release
# Binary: rs/target/release/clx-rust.exe

# Linux
cd rs && cargo build -p capslockx-linux --release
# Binary: rs/target/release/capslockx
```

## Uninstall

### Windows
- **Setup.exe**: Control Panel → Add/Remove Programs → CapsLockX → Uninstall
- **Portable**: run `uninstall.ps1` or delete the folder and remove from PATH

### macOS / Linux
```bash
sudo rm /usr/local/bin/clx
```

### Debian/Ubuntu
```bash
sudo dpkg -r capslockx
```
