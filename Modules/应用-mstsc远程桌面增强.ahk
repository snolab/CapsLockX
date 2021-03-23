if !CapsLockX
    ExitApp
global 上次mstsc窗口hWnd := 0
global 上次CtrlShiftAlt时刻 := 0
global 上次CtrlShiftAlt锁 := 0

; 如果当前操作的远程桌面窗口是全屏窗口，则自动置底，这样可以跟当前电脑桌面上的窗口共同操作
; SetTimer, toggleBottomOrTop, 1
; SetTimer, DetectMSTSC, 1000
Return

setCurrentWindowAsBackground(){
    WinGet 上次mstsc窗口hWnd, id, A
    ; 隐藏，使其失去焦点
    WinHide, ahk_id %上次mstsc窗口hWnd%
    ; 切换焦点到其它窗口
    WinWaitNotActive, ahk_id %上次mstsc窗口hWnd%
    SendEvent !{Esc}
    WinGet, 其它窗口, id, A
    if(!其它窗口){
        其它窗口 := WinExist(".*")
        WinActivate, ahk_id %其它窗口%
    }
    ; 让它显示回来
    WinShow, ahk_id %上次mstsc窗口hWnd%
    ; 然后后置
    WinSet Bottom, , ahk_id %上次mstsc窗口hWnd%
}
DetectMSTSC:
    DetectMSTSC()
return

DetectMSTSC()
{
    msg := ""
    winTitle := "ahk_class TscShellContainerClass ahk_exe mstsc.exe"
    WinWaitActive, % winTitle
    hWnd := WinExist()
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    msg .= "XYWH " X " " Y " " W " " H "`n"
    SysGet, VirtualWidth, 78
    SysGet, VirtualHeight, 79
    msg .= "VWVH " VirtualWidth " " VirtualHeight "`n"

    MonitorIndex := 1
    SysGet, MWA%MonitorIndex%, MonitorWorkArea, %MonitorIndex%
    SX := MWA1Left
    SY := MWA1Top
    SW := MWA1Right - MWA1Left
    SH := MWA1Bottom - MWA1Top
    msg .= "MWA " SX " " SY " " SW " " SH "`n"

    ; Tooltip %X% %Y% %VirtualWidth% %VirtualHeight% %Width% %Height% %A_ScreenWidth% %A_ScreenHeight%
    ; 如果当前操作的远程桌面窗口是全屏窗口，就把它置底
    if (VirtualWidth == W && VirtualHeight == H) {
        ; WinMove, ahk_id %hWnd%, , SX, SY, SW, SH
        ; WinSet Bottom, , ahk_id %hWnd%
        HWND_BOTTOM := 1
        WinRestore, ahk_id %hWnd%
        ; DllCall("SetWindowPos"
        ;, "UInt", hWnd ;handle
        ;, "UInt", HWND_BOTTOM ; z-index
        ;, "Int", X
        ;, "Int", Y
        ;, "Int", W
        ;, "Int", H
        ;, "UInt", SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS) ; SWP_ASYNCWINDOWPOS
        msg .= "MOVE!`n"
    }
    ToolTip %msg%
}
; WinWaitActive ahk_class TscShellContainerClass ahk_exe mstsc.exe

; toggleBottomOrTop:
;     ; 不用担心 SetTimer 消耗 CPU 性能，因为它会在这一步阻塞
;     WinWaitActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
;     WinGet 上次mstsc窗口hWnd
;     WinGet mm, MinMax, ahk_id %上次mstsc窗口hWnd%
;     WinGetPos, X, Y, Width, Height, A
;     SysGet, VirtualWidth, 78
;     SysGet, VirtualHeight, 79
;     ; Tooltip %X% %Y% %VirtualWidth% %VirtualHeight% %Width% %Height% %A_ScreenWidth% %A_ScreenHeight%
;     ; 如果当前操作的远程桌面窗口是全屏窗口，就把它置底
;     if (VirtualWidth == Width && VirtualHeight == Height){
;         WinSet Bottom, , ahk_id %上次mstsc窗口hWnd%
;     }
;     WinWaitNotActive ahk_id %上次mstsc窗口hWnd%
;     Return

; 左右Alt一起按 显示当前mstsc窗口
mstscShow()
{
    TrayTip, , 远程桌面显示, 1
    ; try to 获取当前mstsc窗口
    if !上次mstsc窗口hWnd
        WinGet, 上次mstsc窗口hWnd, , ahk_class TscShellContainerClass ahk_exe mstsc.exe
    WinRestore, ahk_id %上次mstsc窗口hWnd%
    WinSet, Top, , ahk_id %上次mstsc窗口hWnd%
    WinActivate ahk_id %上次mstsc窗口hWnd%
}
mstscHide()
{
    TrayTip, , 远程桌面最小化, 1
    ; 使远程窗口最小化并失去焦点，显示其它窗口
    ; Tooltip % A_PriorHotkey
    ; if (A_PriorHotkey == "<!LCtrl Up" || A_PriorHotkey == "<^LAlt Up" ){
    WinGet, 上次mstsc窗口hWnd
    ; WinMinimizeAllUndo
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
    上次CtrlShiftAlt锁 := 1 ;防止重复运行
    KeyWait, LCtrl
    KeyWait, LAlt
    KeyWait, LShift
    现在 := A_TickCount
    间隔 := 现在 - 上次CtrlShiftAlt时刻
    if(间隔 < 200 && !上次CtrlShiftAlt锁){
        setCurrentWindowAsBackground()
        TrayTip, CapsLockX, 后置当前窗口（主要用于虚拟机和远程桌面）
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
