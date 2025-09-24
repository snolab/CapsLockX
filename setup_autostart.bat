@echo off
echo Setting up CapsLockX to start automatically at Windows login...
echo.

:: Try to create/update the scheduled task (will overwrite if exists)
schtasks /create /tn "CapsLockX_AutoStart" /tr "%~dp0CapsLockX.exe" /sc onlogon /rl highest /f >nul 2>&1

if %errorLevel%==0 (
    echo SUCCESS: CapsLockX will now start automatically when you log in!
    echo (Task has been created/updated in Windows Task Scheduler)
) else (
    echo ERROR: Failed to create the scheduled task.
    echo.
    echo This usually means you need administrator privileges.
    echo Please run this script as Administrator.
)

echo.
pause