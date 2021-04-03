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
global 上次CtrlShiftAlt时刻 := 0
global 上次CtrlShiftAlt锁 := 0
global FLAG_CtrlShiftAlt按下 := 0

Return

setCurrentWindowAsBackground(){
    ; 获取窗口
    WinGet hWnd, id, A
    ; 隐藏
    WinHide, ahk_id %hWnd%
    ; 获取其它窗口
    其它窗口 := WinExist(".*")
    ; 异步后置并显示
    ; HWND_BOTTOM := 1
    ; SWP_SHOWWINDOW := 0x0040
    ; SWP_NOACTIVATE := 0x0010
    ; SWP_ASYNCWINDOWPOS:= 0x4000
    ; SWP_NOMOVE := 0x0002
    ; SWP_NOSIZE := 0x0001
    ; DllCall("SetWindowPos"
    ; , "UInt", hWnd ; handle
    ; , "UInt", HWND_BOTTOM ; z-index
    ; , "Int", 0, "Int", 0, "Int", 0, "Int", 0
    ; , "UInt", SWP_SHOWWINDOW| SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOMOVE | SWP_ASYNCWINDOWPOS)
    WinShow, ahk_id %hWnd%
    WinSet Bottom,, ahk_id %hWnd%
    ; 把焦点给其它窗口
    WinActivate, ahk_id %其它窗口%
    TrayTip, CapsLockX, 后置当前窗口（主要用于虚拟机和远程桌面）
    上次mstsc窗口hWnd := hWnd
}

#if

~$<!<+LCtrl::
~$<^<+LAlt::
~$<!<^LShift::
    FLAG_CtrlShiftAlt按下:=1

return

~$<!<+LCtrl Up::
~$<^<+LAlt Up::
~$<!<^LShift Up::
    if(!(A_PriorKey == "LCtrl" || A_PriorKey == "LAlt" || A_PriorKey == "LShift"))
        Return
    ; 防止重复触发
    if (上次CtrlShiftAlt锁)
        return
    if(!FLAG_CtrlShiftAlt按下)
        return
    FLAG_CtrlShiftAlt按下:=0

    上次CtrlShiftAlt锁 := 1
    ToolTip, 双击 LCtrl LAlt LShift 来最后置当前窗口（主要用于虚拟机和远程桌面）
    KeyWait, LCtrl, T5 ; wait for 5 seconds
    KeyWait, LAlt, T5 ; wait for 5 seconds
    KeyWait, LShift, T5 ; wait for 5 seconds
    SetTimer, MSTSC_ENHANCE_RemoveToolTip, -1024
    现在 := A_TickCount
    间隔 := 现在 - 上次CtrlShiftAlt时刻
    if(间隔 < 200){
        setCurrentWindowAsBackground()
    }else{
        上次CtrlShiftAlt时刻 := 现在
    }
    上次CtrlShiftAlt锁 := 0
return

MSTSC_ENHANCE_RemoveToolTip:
    ToolTip
return
