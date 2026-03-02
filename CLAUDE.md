# CapsLockX – Claude Instructions

## After fixing bugs / building
**Always kill the old process and relaunch the binary after every fix — without being asked:**
```powershell
Stop-Process -Name capslockx -Force -ErrorAction SilentlyContinue
Start-Process 'rs\target\release\capslockx.exe' -WorkingDirectory (Get-Location)
```
Never leave testing to the user when a fresh binary is available.
This applies after EVERY build — including rebases, merges, and any code change.

## Build
```
cargo build -p capslockx-windows --release
```
Binary lands at `rs/target/release/capslockx.exe`.
Kill the running process first (access-denied error = process still running).
