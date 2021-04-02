; ========== CapsLockX ==========
; 名称：远程桌面与虚拟机功能增强
; 描述：提供一种允许远程桌面与当前桌面窗口同时操作的解决方案。
; 版本：v2021.04.02
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

if !CapsLockX
    ExitApp
global 上次mstsc窗口hWnd := 0
global 上次CtrlShiftAlt时刻 := 0
global 上次CtrlShiftAlt锁 := 0

Return

setCurrentWindowAsBackground(){
    WinGet hWnd, id, A
    ; 后置
    WinSet Bottom, , ahk_id %hWnd%
    ; 隐藏，使其失去焦点
    WinHide, ahk_id %hWnd%
    ; 可选切换焦点到其它窗口（现在应该是桌面）
    WinWaitNotActive, ahk_id %hWnd%
    ; SendEvent !+{Esc}
    WinGet, 其它窗口, id, A
    if(0)
        ToolTip %其它窗口%
    if(!其它窗口){
        其它窗口 := WinExist(".*")
        WinActivate, ahk_id %其它窗口%
    }
    WinGetTitle, t, ahk_id %其它窗口%
    WinGetClass, cc,ahk_id %其它窗口%
    if(0)
        Tooltip %其它窗口% - %t% %c%
    TrayTip, CapsLockX, 后置当前窗口（主要用于虚拟机和远程桌面）
    ; 让它显示回来
    WinShow, ahk_id %hWnd%
    上次mstsc窗口hWnd := hWnd
}

DetectMSTSC:
    DetectMSTSC()
return

DetectMSTSC(){
    msg := ""
    winTitle := "ahk_class TscShellContainerClass ahk_exe mstsc.exe"
    WinWaitActive, % winTitle
    hWnd := WinExist()
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    msg .= "XYWH " X " " Y " " W " " H "`n"
    SysGet, VirtualWidth, 78
    SysGet, VirtualHeight, 79
    msg .= "VWVH " VirtualWidth " " VirtualHeight "`n"
    ; 
    MonitorIndex := 1
    SysGet, MWA%MonitorIndex%, MonitorWorkArea, %MonitorIndex%
    SX := MWA1Left
    SY := MWA1Top
    SW := MWA1Right - MWA1Left
    SH := MWA1Bottom - MWA1Top
    msg .= "MWA " SX " " SY " " SW " " SH "`n"
    if(0)
        Tooltip %X% %Y% %VirtualWidth% %VirtualHeight% %Width% %Height% %A_ScreenWidth% %A_ScreenHeight%
    ; 如果当前操作的远程桌面窗口是全屏窗口，就把它置底
    if (VirtualWidth == W && VirtualHeight == H){
        HWND_BOTTOM := 1
        WinRestore, ahk_id %hWnd%
        msg .= "MOVE!`n"
    }
    if(0)
        ToolTip %msg%
}
; 左右Alt一起按 显示当前mstsc窗口
mstscShow(){
    TrayTip, , 远程桌面显示, 1
    if !上次mstsc窗口hWnd
        WinGet, 上次mstsc窗口hWnd, , ahk_class TscShellContainerClass ahk_exe mstsc.exe
    WinRestore, ahk_id %上次mstsc窗口hWnd%
    WinSet, Top, , ahk_id %上次mstsc窗口hWnd%
    WinActivate ahk_id %上次mstsc窗口hWnd%
}
mstscHide(){
    TrayTip, , 远程桌面最小化, 1
    ; 使远程窗口最小化并失去焦点，显示其它窗口
    WinGet, 上次mstsc窗口hWnd
    ; 后置
    WinSet Bottom, , ahk_id %上次mstsc窗口hWnd%
    ; 最小化
    WinMinimize ahk_id %上次mstsc窗口hWnd%
    ; 使其失去焦点
    WinHide, ahk_id %上次mstsc窗口hWnd%
    ; WinRestore, ahk_id %上次mstsc窗口hWnd%
    WinShow, ahk_id %上次mstsc窗口hWnd%
}

#if

~$<!<+LCtrl Up::
~$<^<+LAlt Up::
~$<!<^LShift Up::
    if(!(A_PriorKey == "LCtrl" || A_PriorKey == "LAlt" || A_PriorKey == "LShift"))
        Return
    ; 防止重复运行
    if (上次CtrlShiftAlt锁)
        return
    上次CtrlShiftAlt锁 := 1
    ToolTip, 双击 LCtrl LAlt LShift 来最后置当前窗口（主要用于虚拟机和远程桌面） %上次CtrlShiftAlt时刻%
    KeyWait, LCtrl
    KeyWait, LAlt
    KeyWait, LShift
    现在 := A_TickCount
    间隔 := 现在 - 上次CtrlShiftAlt时刻
    if(间隔 < 200){
        setCurrentWindowAsBackground()
    }else{
        上次CtrlShiftAlt时刻 := 现在
    }
    上次CtrlShiftAlt锁 := 0
Return

<!RAlt Up:: mstscShow()
>!LAlt Up::mstscShow()

#If WinActive("ahk_class TscShellContainerClass ahk_exe mstsc.exe")

; 左右Alt或Ctrl一起按 显示当前mstsc窗口
<!RAlt Up:: mstscHide()
>!LAlt Up::mstscHide()
