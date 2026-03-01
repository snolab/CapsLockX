# CapsLockX – Claude Instructions

## After fixing bugs / building
**Always kill the old process and relaunch the binary after every fix:**
```powershell
Stop-Process -Name capslockx -Force -ErrorAction SilentlyContinue
Start-Process 'rs\target\release\capslockx.exe' -WorkingDirectory (Get-Location)
```
Never leave testing to the user when a fresh binary is available.

## Build
```
cargo build -p capslockx-windows --release
```
Binary lands at `rs/target/release/capslockx.exe`.
Kill the running process first (access-denied error = process still running).
