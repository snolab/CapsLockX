; ========== CapsLockX ==========
; 名称：CapsLockX 核心
; 描述：用于处理配置文件、加载其它模块、提供基本 CapsLockX 键触发功能
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 编码：UTF-8 with BOM
; 版权：Copyright © 2017-2024 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========
; 创建：Snowstar QQ: 997596439
; 参与完善：张工 QQ: 45289331

Process Priority, , High ; 脚本高优先级
SetTitleMatchMode RegEx
; #NoEnv ; 不检查空变量是否为环境变量

#SingleInstance Force ; 跳过对话框并自动替换旧实例（在启动成功后有效）
; if(A_IsAdmin){
;     #SingleInstance Force ; 跳过对话框并自动替换旧实例（在启动成功后有效）
; }else{
;     #SingleInstance, Off ; 允许多开，然后在下面用热键把其它实例关掉
; }
#Persistent
#MaxHotkeysPerInterval 1000 ; 时间内按键最大次数（通常是一直按着键触发的。。）
#InstallMouseHook ; 安装鼠标钩子

; 载入设定
; #Include %A_ScriptDir%/Core/CapsLockX-Config.ahk
#Include %A_ScriptDir%/../Core/CapsLockX-Config.ahk

; 模式处理
global CapsLockX := 1 ; 模块运行标识符
global CapsLockXMode := 0
global ModuleState := 0
global CLX_FnActed := 0
global CM_NORMAL := 0 ; 普通模式（键盘的正常状态）
global CM_FN := 1 ; 组合键 CapsLockX 模式（或称组合键模式
global CM_CapsLockX := 2 ; CapsLockX 模式，通过长按CLX键进入
; global CM_FNX := 3 ; FnX 模式并不存在
global CapsLockPressTimestamp := 0
global CLX_上次触发键 := ""

; ── Rust IPC: --no-core mode ──────────────────────────────────────────────────
; When launched with --no-core, Rust handles the keyboard hook and publishes
; mode via shared memory "CapsLockX_SharedState".  AHK reads it on a 10ms timer.
global CLX_NoCore := !!RegExMatch(DllCall("GetCommandLine", "str"), "--no-core")
global CLX_ShmPtr := 0
global CLX_ShmHandle := 0

if (CLX_NoCore) {
    ; --no-core is a hard flag: Rust owns the keyboard hook regardless of whether
    ; shared memory is available.  Shared memory is optional (used only for mode sync).
    CLX_InitSharedMemory()
}

; value func
CapsLockX()
{
    return CapsLockX
}
CapsLockXMode()
{
    return CapsLockXMode
}
; 根据灯的状态来切换到上次程序退出时使用的模式（不）
UpdateCapsLockXMode()
{
    if (T_UseCapsLockLight) {
        CapsLockXMode := GetKeyState("CapsLock", "P")
    }
    if (T_UseScrollLockLight) {
        CapsLockXMode |= GetKeyState("ScrollLock", "T") << 1
    }
    Return CapsLockXMode
}
UpdateCapsLockXMode()

; 根据当前模式，切换灯
; When Rust core manages the tray, hide AHK's tray icon to avoid duplicates.
if (CLX_NoCore) {
    Menu, tray, NoIcon
} else {
    Menu, tray, icon, %T_SwitchTrayIconOff%
}
UpdateCapsLockXLight()

global T_IgnoresByLines
defaultIgnoreFilePath := "./Data/CapsLockX.defaults.ignore.txt"
userIgnoreFilePath := CLX_ConfigDir "/CapsLockX.user.ignore.txt"
FileRead, T_IgnoresByLines, %userIgnoreFilePath%
if (!T_IgnoresByLinesUser) {
    FileCopy, %defaultIgnoreFilePath%, %userIgnoreFilePath%
    FileRead, T_IgnoresByLines, %userIgnoreFilePath%
}

global CLX_Paused := 0

#if CLX_Avaliable()

#if CLX_NotAvaliable()

#If

; In --no-core mode Rust owns the keyboard hook; skip AHK hotkey registration.
if (!CLX_NoCore) {
    Hotkey, If, CLX_Avaliable()

    if(T_XKeyAsCapsLock)
        Hotkey *CapsLock, CLX_Dn
    if(T_XKeyAsSpace)
        Hotkey *Space, CLX_Dn
    if(T_XKeyAsInsert)
        Hotkey *Insert, CLX_Dn
    if(T_XKeyAsScrollLock)
        Hotkey *ScrollLock, CLX_Dn
    if(T_XKeyAsRAlt)
        Hotkey *RAlt, CLX_Dn

    Hotkey, If, CLX_NotAvaliable()

    if(T_XKeyAsCapsLock)
        Hotkey CapsLock, CLX_NotAvaliable
    if(T_XKeyAsSpace)
        Hotkey Space, CLX_NotAvaliable
    if(T_XKeyAsInsert)
        Hotkey Insert, CLX_NotAvaliable
    if(T_XKeyAsScrollLock)
        Hotkey ScrollLock, CLX_NotAvaliable
    if(T_XKeyAsRAlt)
        Hotkey RAlt, CLX_NotAvaliable

    Hotkey, If

    if(T_XKeyAsCapsLock)
        Hotkey *CapsLock Up, CLX_Up
    if(T_XKeyAsSpace)
        Hotkey *Space Up, CLX_Up
    if(T_XKeyAsInsert)
        Hotkey *Insert Up, CLX_Up
    if(T_XKeyAsScrollLock)
        Hotkey *ScrollLock Up, CLX_Up
    if(T_XKeyAsRAlt)
        Hotkey *RAlt Up, CLX_Up
}

SetWorkingDir, %A_ScriptDir%\..\

#Include Core\CapsLockX-i18n.ahk
; todo: move this generated file into user folder
#Include Core\CapsLockX-ModulesRunner.ahk
CLX_Loaded()
#Include Core\CapsLockX-ModulesFunctions.ahk

SetWorkingDir, %A_ScriptDir%\..\

#Include Core\CapsLockX-RunSilent.ahk

#Include Core\CapsLockX-QuickTips.ahk

; All modules loaded, hotkeys and keyboard hook are now installed.
; Signal the Rust launcher so it can install its own hook after ours.
if (CLX_NoCore)
    CLX_SignalAhkReady()

#If

UpdateCapsLockXLight()
{
    NowLightState := ((CapsLockXMode & CM_CapsLockX) || (CapsLockXMode & CM_FN))
    static LastLightState := NowLightState

    ; notice
    static LastCapsLockXMode := CapsLockXMode
    if (!(LastCapsLockXMode & CM_CapsLockX) && (CapsLockXMode & CM_CapsLockX)) {
        ToolTip 进入CLX模式，可单击任意CLX键退出
        SetTimer CLX_HideToolTips, -1000
    }
    if ((LastCapsLockXMode & CM_CapsLockX) && !(CapsLockXMode & CM_CapsLockX)) {
        ToolTip 退出CLX模式，可长按CLX键或按CapsLock+Space键，再次进入
        SetTimer CLX_HideToolTips, -1000
    }
    LastCapsLockXMode := CapsLockXMode

    IsEdge := !(NowLightState == LastLightState)
    if (!IsEdge) {
        Return
    }
    UpEdge := NowLightState && !LastLightState

    if (T_UseScrollLockLight && GetKeyState("ScrollLock", "T") != NowLightState) {
        Send {ScrollLock}
    }
    if (T_UseCursor) {
        ; Module
        try {
            Func("UpdateCapsCursor").Call(NowLightState)
        }
    }
    if (UpEdge) {
        if (!CLX_NoCore) {
            global T_SwitchTrayIconOn
            Menu, tray, icon, %T_SwitchTrayIconOn%
        }
        if (T_SwitchSound && T_SwitchSoundOn) {
            SoundPlay %T_SwitchSoundOn%
        }
    } else {
        if (!CLX_NoCore) {
            global T_SwitchTrayIconOff
            Menu, tray, icon, %T_SwitchTrayIconOff%
        }
        if (T_SwitchSound && T_SwitchSoundOff) {
            SoundPlay %T_SwitchSoundOff%
        }
    }
    LastLightState := NowLightState
}
CapsLockXTurnOff()
{
    CapsLockXMode &= ~CM_CapsLockX
    re := UpdateCapsLockXLight()
    Return re
}
CapsLockXTurnOn()
{
    CapsLockXMode |= CM_CapsLockX
    re := UpdateCapsLockXLight()
    Return re
}
CLX_NotAvaliable()
{
    return !CLX_Avaliable()
}
CLX_Avaliable()
{
    return 1
    flag_IgnoreWindow := 0
    loop, Parse, T_IgnoresByLines, `n, `r
    {
        content := Trim(RegExReplace(A_LoopField, "^#.*", ""))
        if (content) {
            ; flag_IgnoreWindow := flag_IgnoreWindow || WinActive(content)
            if (WinActive(content)) {
                ToolTip, ignored by %content%
                return false
            }
        }
    }
    return !CLX_Paused
}
CLX_Loaded()
{
    ; 使用退出键退出其它实例
    SendInput ^!+\

    TrayTip CapsLockX %CLX_VersionName%, % t("加载成功")
    Menu, Tray, Tip, CapsLockX %CLX_VersionName%
}
CLX_Reload()
{
    ToolTip, % t("CapsLockX 重载中")
    static times := 0
    times += 1
    if (times == 1 && false) {
        ;;感觉limited user 不太管用
        ; 使用 RunAsLimitiedUser 避免重载时出现 Could not close the previous instance of this script.  Keep waiting?
        RunAsLimitiedUser(A_WorkingDir "\CapsLockX.exe", A_WorkingDir)
        ; 这里启动新实例后不用急着退出当前实例
        ; 如果重载的新实例启动成功，则会自动使用热键结束掉本实例
        ; 而如果没有启动成功则保留本实例，以方便修改语法错误的模块
    } else {
        ; 但如果用户多次要求重载，那就直接退出掉好了（即，双击重载会强制退出当前实例）
        RunAsSameUser(A_WorkingDir "\CapsLockX.exe", A_WorkingDir)
        ExitApp
    }
}

CLX_ModeExit()
{
    ; TrayTip CapsLockX, 退出CLX模式
    ; ToolTip 退出CLX模式
    ; SetTimer CLX_HideToolTips, -1000
    CapsLockXMode &= ~CM_CapsLockX
}
CLX_ModeEnter()
{
    ; ToolTip 进入CLX模式
    ; SetTimer CLX_HideToolTips, -1000
    CapsLockXMode |= CM_CapsLockX
}

CLX_Dn()
{
    /*
    组合键细节：
    模式：
    普通模式 + 按住CLX -> CLX模式
    普通模式 + 长按CLX -> CLX锁定模式
    普通模式 + (CLX+SPACE) -> CLX锁定模式
    CLX模式 + 弹起CLX -> 普通模式
    CLX锁定模式 + 按住CLX -> CLX模式

    CapsLock + Space 同时按下：进入CLX模式
    CLX长按：进入CLX锁定模式
    CLX单击：退出CLX锁定模式

    */
    ; 按住其它键的时候 不触发 CapsLockX 避免影响打字
    CLX_上次触发键 := 触发键 := RegExReplace(A_ThisHotkey, "[\$\*\!\^\+\#\s]")
    其它键按住 := 触发键 && 触发键 != A_PriorKey && GetKeyState(A_PriorKey, "P")
    WheelQ := InStr("WheelDown|WheelUp", A_PriorKey)
    SpaceQ := 触发键 == "Space"
    CapsLockQ := 触发键 == "CapsLock"
    ModifierQ := InStr("LControl|RControl|LShift|RShift|LAlt|RAlt|LWin|RWin", A_PriorKey)
    ModifierEnableQ := !SpaceQ && ModifierQ

    CLX_AND_SPACE_Q := (A_PriorKey == "CapsLock" && 触发键 == "Space") || (触发键 == "CapsLock" && A_PriorKey == "Space" )
    if (CLX_AND_SPACE_Q && A_TimeSincePriorHotkey < 250) {
        ; CLX_ModeEnter()
        CapsLockXMode |= CM_CapsLockX
        UpdateCapsLockXLight()
        KeyWait %触发键%
        return
    }

    ; tooltip % ModifierQ "a" ModifierEnableQ "a" WheelQ "a"  其它键按住
    BypassCapsLockX := !ModifierEnableQ && !WheelQ && 其它键按住
    if (BypassCapsLockX) {
        CLX_上次触发键 := ""
        ; ToolTip, % first5char "_" 触发键
        Send {Blind}{%触发键% Down}
        KeyWait %触发键%
        Send {Blind}{%触发键% Up}
        Return
    }
    ; 记录 CapsLockX 按住的时间
    if ( CapsLockPressTimestamp == 0) {
        CapsLockPressTimestamp := A_TickCount
    }
    ; 进入 Fn 模式
    ; if (CapsLockXMode & CM_CapsLockX) {
    ;     CLX_ModeExit()
    ;     KeyWait, %waitKey% ; wait to prevent flashing the quit and enter message
    ; }
    ; CLX_ModeExit()

    CapsLockXMode |= CM_FN
    CapsLockXMode &= ~CM_CapsLockX

    ; ToolTip clxmode
    if (A_PriorKey == CLX_上次触发键) {
        if (A_PriorKey == "Space") {
            ; 长按空格时保持原功能
            ; TODO: read system repeat interval
            if ( A_TickCount - CapsLockPressTimestamp > 200) {
                Send, {Blind}{Space}
            }
        } else {
            if ( A_TickCount - CapsLockPressTimestamp > 1000) {
                ; (20210817)长按（空格除外）
                waitKey := CLX_上次触发键
                ; 取消长按CLX进入CLX锁定模式
                ; CLX_ModeEnter()
                ; 尝试增加长按显示热键提示
                ; Func("CLX_LongPressDown").Call()
                KeyWait, %waitKey% ; wait to prevent flashing the quit and enter message
                ; Func("CLX_LongPressUp").Call()
            }
        }
    }
    UpdateCapsLockXLight()
}
CLX_Up()
{
    CapsLockPressTimestamp := 0

    ; CLX弹起时退出 Fn 模式
    CapsLockXMode &= ~CM_FN

    ; CLX单击弹起时
    if (A_PriorKey == CLX_上次触发键) {
        if (CapsLockXMode & CM_CapsLockX) {
            ; CLX_ModeExit()
        } else {
            ; 单击 CapsLockX
            if (CLX_上次触发键 == "CapsLock") {
                ; 切换 CapsLock 状态（原功能）
                if (GetKeyState("CapsLock", "T")) {
                    SetCapsLockState, Off
                } else {
                    SetCapsLockState, On
                }
            }
            ; 单击 空格键
            if (CLX_上次触发键 == "Space") {
                ; 原功能（按空格键）
                Send {Blind}{Space}
            }
        }
    }
    UpdateCapsLockXLight()
    CLX_上次触发键 := ""
}
RunAsSameUser(CMD, WorkingDir)
{
    Run, %CMD%, %WorkingDir%
}

RunAsLimitiedUser(CMD, WorkingDir)
{
    ; ref: [Run as normal user (not as admin) when user is admin - Ask for Help - AutoHotkey Community]( https://autohotkey.com/board/topic/79136-run-as-normal-user-not-as-admin-when-user-is-admin/ )
    ;
    ; TEST DEMO
    ; schtasks /Create /tn CLX_RunAsLimitedUser /sc ONCE /tr "cmd /k cd \"C:\\users\\snomi\\\" && notepad \".\\tmp.txt\"" /F /ST 00:00
    ; schtasks /Run /tn CLX_RunAsLimitedUser
    ; schtasks /Delete /tn CLX_RunAsLimitedUser /F
    ;
    ; Safe_WorkingDir := RegExReplace("C:\users\snomi\", "\\", "\\")
    ; Safe_CMD := RegExReplace(RegExReplace("notepad "".\temp.txt""", "\\", "\\"), "\""", "\""")
    Safe_WorkingDir := RegExReplace(WorkingDir, "\\", "\\")
    Safe_CMD := RegExReplace(RegExReplace(CMD, "\\", "\\"), "\""", "\""")
    RunWait cmd /c schtasks /Create /tn CLX_RunAsLimitedUser /F /sc ONCE /ST 00:00 /tr "cmd /c cd \"%Safe_WorkingDir% && %Safe_CMD%\", , Hide
    RunWait cmd /c schtasks /Run /tn CLX_RunAsLimitedUser, , Hide
    RunWait cmd /c schtasks /Delete /tn CLX_RunAsLimitedUser /F, , Hide
}
; 接下来是流程控制
#if

; CapsLockX Mode switching processing
CLX_NotAvaliable:
    TrayTip, CapsLockX, NotAvaliable
Return

CLX_HideToolTips()
{
    ToolTip
    SetTimer CLX_HideToolTips, Off
}

; ── Shared memory IPC helpers ─────────────────────────────────────────────────

CLX_SignalAhkReady()
{
    ; EVENT_MODIFY_STATE = 0x0002
    hEvent := DllCall("OpenEventW", "UInt", 0x0002, "Int", 0, "WStr", "CapsLockX_AhkReady", "Ptr")
    if (hEvent) {
        DllCall("SetEvent", "Ptr", hEvent)
        DllCall("CloseHandle", "Ptr", hEvent)
    }
}

CLX_InitSharedMemory()
{
    ; Open the shared memory region created by the Rust core.
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
        UpdateCapsLockXLight()
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
