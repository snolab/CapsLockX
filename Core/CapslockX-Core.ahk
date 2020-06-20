; UTF-8 with BOM
;
; 程序核心
; 最后更新：(20190707)
;
; Copyright © 2017-2019 snomiao@gmail.com
; 创建：Snowstar QQ: 997596439
; 参与完善：张工 QQ: 45289331
; LICENCE: GNU GPLv3
;

Process Priority, , High ; 脚本高优先级
SetTitleMatchMode RegEx
#SingleInstance Force ; 跳过对话框并自动替换旧实例
; #NoTrayIcon ; 隐藏托盘图标
; #NoEnv ; 不检查空变量是否为环境变量
#Persistent
#MaxHotkeysPerInterval 1000 ; 时间内按键最大次数（通常是一直按着键触发的）
#InstallMouseHook ; 安装鼠标钩子

; 载入设定
#Include CapslockX-Settings.ahk

AskRunAsAdmin()
{
    full_command_line := DllCall("GetCommandLine", "str")
    if (!A_IsAdmin And !RegExMatch(full_command_line, " /restart(?!\S)")) {
        try {
            if A_IsCompiled {
                
                Run *RunAs "%A_ScriptFullPath%" /restart, "%A_WorkingDir%"
            }
            Else{
                Run *RunAs "%A_AhkPath%" /restart "%A_ScriptFullPath%", "%A_WorkingDir%"
            }
        }
        ExitApp
    }
}

If(T_AskRunAsAdmin) {
    AskRunAsAdmin()
}

; 模式处理
global CapslockX := 1 ; 模块运行标识符
global CapslockXMode := 0
global ModuleState := 0
global CapslockX_FnActed := 0
global CM_NORMAL := 0 ; 普通模式
global CM_FN := 1 ; 临时 CapslockX 模式
global CM_CapslockX := 2 ; CapslockX 模式
; global CM_FNX := 3 ; FnX 模式
global LastLightState := ((CapslockXMode & CM_CapslockX) || (CapslockXMode & CM_FN))
global globalHelpInfo := ""

CapslockAddHelp(helpStr)
{
    globalHelpInfo .= helpStr . "`n"
}
CapslockShowHelp(helpStr){
    ToolTip % helpStr
    KeyWait, /
    ToolTip
}


; 切换模式
UpdateCapslockXMode()
{
    CapslockXMode := GetKeyState(T_CapslockXKey, "P")
    If(T_UseScrollLockLight) {
        CapslockXMode |= GetKeyState("ScrollLock", "T") << 1
    }
    
    Return CapslockXMode
}
UpdateCapslockXMode()
; 根据当前模式，切换灯
Menu, tray, icon, ./数据/图标白.ico
UpdateLight()
{
    NowLightState := ((CapslockXMode & CM_CapslockX) || (CapslockXMode & CM_FN))
    ; UpdateCapsCursor(1)
    if (NowLightState == LastLightState) {
        Return
    }
    ; ToolTip testDDDD
    
    if ( NowLightState && !LastLightState) {
        Menu, tray, icon, ./数据/图标蓝.ico
        if (T_SwitchSound && T_SwitchSoundOn) {
            SoundPlay %T_SwitchSoundOn%
        }
    }
    if ( !NowLightState && LastLightState ) {
        Menu, tray, icon, ./数据/图标白.ico
        if (T_SwitchSound && T_SwitchSoundOff) {
            SoundPlay %T_SwitchSoundOff%
        }
    }
    if (T_UseScrollLockLight) {
        ; ToolTip % CapslockXMode
        if (GetKeyState("ScrollLock", "T") != NowLightState) {
            Send {ScrollLock}
            Return 1
        }
    }
    if (1 || T_UseCursor) {
        ; ToolTip test
        UpdateCapsCursor(NowLightState)
    }
    
    ; tips(CapslockXMode)
    LastLightState := NowLightState
}

CapslockXTurnOff()
{
    CapslockXMode &= ~CM_CapslockX
    re =: UpdateLight()
    Return re
}
CapslockXTurnOn()
{
    CapslockXMode |= CM_CapslockX
    re =: UpdateLight()
    Return re
}

Hotkey *%T_CapslockXKey%, CapslockX_Dn
Hotkey *%T_CapslockXKey% Up, CapslockX_Up

#Include Core\CapslockX-LoadModules.ahk

#If
    ; CapslockX模式切换
CapslockX_Dn:
    ; 进入 Fn 模式
    CapslockXMode |= CM_FN
    ; 限制在远程桌面里无法进入 Fn 模式，避免和远程桌面里的 CapslockX 冲突
    if (WinActive("ahk_class TscShellContainerClass ahk_exe mstsc.exe")) {
        CapslockXMode &= ~CM_FN
    }
    
    UpdateLight()
Return

CapslockX_Up:
    ; 退出 Fn 模式
    CapslockXMode &= ~CM_FN
    ; 规避 Fn 功能键
    CapslockX_FnActed := CapslockX_FnActed || (A_PriorKey != T_CapslockXKey && A_PriorKey != "Insert")
    if (!CapslockX_FnActed) {
        CapslockXMode ^= CM_CapslockX
        
        ; 限制在远程桌面里无法进入 CapslockX 模式，避免和远程桌面里的 CapslockX 冲突
        if (WinActive("ahk_class TscShellContainerClass ahk_exe mstsc.exe")) {
            CapslockXMode &= ~CM_CapslockX
        }
    }
    ;ToolTip, %CapslockX_FnActed%
    CapslockX_FnActed := 0
    UpdateLight()
Return

;
; #If T_CapslockXKey == "CapsLock"
; !CapsLock:: CapsLock ;


#If CapslockXMode

; 显示使用方法
; /::
;     ToolTip % globalHelpInfo
;     KeyWait, /
;     Return
; / Up:: ToolTip
/:: CapslockShowHelp(globalHelpInfo)



; 用ScrollLock代替Capslock键
#if T_UseScrollLockAsCapslock
    $ScrollLock:: CapsLock

#if T_UseDoubleClickShiftAsCapslock
    ; TODO

#if

; 软重启键
!F12:: Reload

; 硬重启键
^!F12::
    ; Run CapslockX.ahk, %A_WorkingDir%
    Run CapslockX.exe, %A_WorkingDir%
    ExitApp
Return

; 结束键
^!+F12:: ExitApp

*Insert:: GoSub CapslockX_Dn
*Insert Up:: GoSub CapslockX_Up

