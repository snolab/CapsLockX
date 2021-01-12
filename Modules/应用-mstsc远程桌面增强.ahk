if !CapsLockX
    ExitApp
global last_mstsc := 0

; 如果当前操作的远程桌面窗口是全屏窗口，则自动置底，这样可以跟当前电脑桌面上的窗口共同操作
; SetTimer, toggleBottomOrTop, 1
; SetTimer, DetectMSTSC, 1000
Return

DetectMSTSC:
    DetectMSTSC()
return

DetectMSTSC()
{
    msg := ""
    winTitle :=  "ahk_class TscShellContainerClass ahk_exe mstsc.exe"
    WinWaitActive, % winTitle
    hWnd := WinExist()
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    msg .= "XYWH " X " " Y " " W " " H "`n"
    SysGet, VirtualWidth, 78
    SysGet, VirtualHeight, 79
    msg .= "VWVH " VirtualWidth  " " VirtualHeight "`n"
    
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
;     WinGet last_mstsc
;     WinGet mm, MinMax, ahk_id %last_mstsc%
;     WinGetPos, X, Y, Width, Height, A
;     SysGet, VirtualWidth, 78
;     SysGet, VirtualHeight, 79
;     ; Tooltip %X% %Y% %VirtualWidth% %VirtualHeight% %Width% %Height% %A_ScreenWidth% %A_ScreenHeight%
;     ; 如果当前操作的远程桌面窗口是全屏窗口，就把它置底
;     if (VirtualWidth == Width && VirtualHeight == Height){
;         WinSet Bottom, , ahk_id %last_mstsc%
;     }
;     WinWaitNotActive ahk_id %last_mstsc%
;     Return

; 左右Alt一起按 显示当前mstsc窗口
mstscShow()
{
    TrayTip, , 远程桌面显示, 1
    ; try to 获取当前mstsc窗口
    if !last_mstsc
        WinGet, last_mstsc, , ahk_class TscShellContainerClass ahk_exe mstsc.exe
    WinRestore, ahk_id %last_mstsc%
    WinSet, Top, , ahk_id %last_mstsc%
    WinActivate ahk_id %last_mstsc%
}
mstscHide()
{
    
    TrayTip, , 远程桌面最小化, 1
    ; 使远程窗口最小化并失去焦点，显示其它窗口
    ; Tooltip % A_PriorHotkey
    ; if (A_PriorHotkey == "<!LCtrl Up" || A_PriorHotkey == "<^LAlt Up" ){
    WinGet, last_mstsc
    ; WinMinimizeAllUndo
    ; 后置
    WinSet Bottom, , ahk_id %last_mstsc%
    ; 最小化
    WinMinimize ahk_id %last_mstsc%
    ; 使其失去焦点
    WinHide, ahk_id %last_mstsc%
    ; WinRestore, ahk_id %last_mstsc%
    WinShow, ahk_id %last_mstsc%
    
}

#if
    
<!RAlt Up:: mstscShow()
>!LAlt Up::mstscShow()

#IfWinActive ahk_class TscShellContainerClass ahk_exe mstsc.exe

; 左右Alt或Ctrl一起按 显示当前mstsc窗口
<!RAlt Up:: mstscHide()
>!LAlt Up::mstscHide()
