@echo off
REM Dev loop for Windows: watch source, incrementally rebuild, copy to root, restart clx.
REM Wraps dev-clx.ps1 (cargo watch -> build.ps1). Just run `dev` from this folder.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0dev-clx.ps1" %*
