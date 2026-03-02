; ========== CapsLockX ==========
; ModuleLoader.ahk — Lightweight AHK module host spawned by Rust core
; Provides: config, i18n, shared memory IPC, mode globals, module loading
; Does NOT: compile README, check updates, copy EXE, manage tray icons
; Encoding: UTF-8 with BOM
; ========== CapsLockX ==========

#SingleInstance Force
#Persistent
#NoTrayIcon
#MaxHotkeysPerInterval 1000
#InstallMouseHook

Process Priority, , High
SetTitleMatchMode RegEx
SetWorkingDir, %A_ScriptDir%\..\

; ── Config & i18n ─────────────────────────────────────────────────────────────
#Include %A_ScriptDir%\CapsLockX-Config.ahk
#Include %A_ScriptDir%\CapsLockX-i18n.ahk

; ── Globals that modules depend on ────────────────────────────────────────────
global CapsLockX := 1
global CapsLockXMode := 0
global ModuleState := 0
global CLX_FnActed := 0
global CM_NORMAL := 0
global CM_FN := 1
global CM_CapsLockX := 2
global CapsLockPressTimestamp := 0
global CLX_Paused := 0
global CLX_ModuleDir := "./Modules"
global CLX_CoreDir := "./Core"
global CLX_Version
FileRead, CLX_Version, ./Core/version.txt
CLX_Version := CLX_Version ? Trim(CLX_Version, "`r`n`t ") : "0.0.0"
global CLX_VersionName := "v" CLX_Version

; ── Ignore list (used by CLX_Avaliable) ───────────────────────────────────────
global T_IgnoresByLines
defaultIgnoreFilePath := "./Data/CapsLockX.defaults.ignore.txt"
userIgnoreFilePath := CLX_ConfigDir "/CapsLockX.user.ignore.txt"
FileRead, T_IgnoresByLines, %userIgnoreFilePath%
if (!T_IgnoresByLines) {
    FileCopy, %defaultIgnoreFilePath%, %userIgnoreFilePath%
    FileRead, T_IgnoresByLines, %userIgnoreFilePath%
}

; ── Shared memory IPC with Rust core ──────────────────────────────────────────
global CLX_NoCore := true  ; always true — Rust is the core
global CLX_ShmPtr := 0
global CLX_ShmHandle := 0

if (!CLX_InitSharedMemory()) {
    ; Shared memory not available — exit, Rust isn't running
    MsgBox ModuleLoader: shared memory IPC failed. Is the Rust core running?
    ExitApp
}

; ── Context functions for #if directives ──────────────────────────────────────
#if CLX_Avaliable()

#if CLX_NotAvaliable()

#If

; ── Load generated module files ───────────────────────────────────────────────
#Include Core\CapsLockX-ModulesRunner.ahk
CLX_Loaded()
#Include Core\CapsLockX-ModulesFunctions.ahk

SetWorkingDir, %A_ScriptDir%\..\

#Include Core\CapsLockX-RunSilent.ahk
#Include Core\CapsLockX-QuickTips.ahk

#If

; ── Stubs & helpers ───────────────────────────────────────────────────────────

UpdateCapsLockXLight()
{
    ; No-op — Rust handles tray icon updates via shared memory
}

CapsLockX()
{
    return CapsLockX
}
CapsLockXMode()
{
    return CapsLockXMode
}

CLX_Avaliable()
{
    return !CLX_Paused
}
CLX_NotAvaliable()
{
    return !CLX_Avaliable()
}

CLX_Loaded()
{
    ; Lightweight loaded notification — no hotkey-based instance kill
    TrayTip CapsLockX %CLX_VersionName%, Modules loaded (Rust core)
}

CLX_Reload()
{
    Reload
}

CapsLockXTurnOff()
{
    CapsLockXMode &= ~CM_CapsLockX
}
CapsLockXTurnOn()
{
    CapsLockXMode |= CM_CapsLockX
}

CLX_ModeExit()
{
    CapsLockXMode &= ~CM_CapsLockX
}
CLX_ModeEnter()
{
    CapsLockXMode |= CM_CapsLockX
}

CLX_HideToolTips()
{
    ToolTip
    SetTimer CLX_HideToolTips, Off
}

; ── Shared memory IPC helpers ─────────────────────────────────────────────────

CLX_InitSharedMemory()
{
    FILE_MAP_READ := 0x0004
    hMap := DllCall("OpenFileMappingW", "UInt", FILE_MAP_READ, "Int", 0, "WStr", "CapsLockX_SharedState", "Ptr")
    if (!hMap) {
        return false
    }
    ptr := DllCall("MapViewOfFile", "Ptr", hMap, "UInt", FILE_MAP_READ, "UInt", 0, "UInt", 0, "UPtr", 256, "Ptr")
    if (!ptr) {
        DllCall("CloseHandle", "Ptr", hMap)
        return false
    }
    ; Verify protocol version
    version := NumGet(ptr + 0, 0, "UInt")
    if (version != 1) {
        DllCall("UnmapViewOfFile", "Ptr", ptr)
        DllCall("CloseHandle", "Ptr", hMap)
        return false
    }
    CLX_ShmPtr := ptr
    CLX_ShmHandle := hMap
    SetTimer, CLX_ReadSharedMemory, 10
    OnExit("CLX_CleanupShm")
    return true
}

CLX_ReadSharedMemory:
    if (CLX_ShmPtr) {
        CapsLockXMode := NumGet(CLX_ShmPtr + 0, 4, "UInt")
    }
Return

CLX_CleanupShm()
{
    SetTimer, CLX_ReadSharedMemory, Off
    if (CLX_ShmPtr) {
        DllCall("UnmapViewOfFile", "Ptr", CLX_ShmPtr)
        CLX_ShmPtr := 0
    }
    if (CLX_ShmHandle) {
        DllCall("CloseHandle", "Ptr", CLX_ShmHandle)
        CLX_ShmHandle := 0
    }
}
