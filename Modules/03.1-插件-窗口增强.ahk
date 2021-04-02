; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：窗口增强模块
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.07.04
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; 许可证 LICENCE: GNU GPLv3 ( https://www.gnu.org/licenses/gpl-3.0.html )
; ========== CapsLockX ==========
;
; Exit if running without CapsLockX
;
if !CapsLockX
    ExitApp

; #If WinActive("ahk_class MultitaskingViewFrame")
; #IfWinActive ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe
; #If

CapsLockX_AppendHelp("
(
窗口增强
| 作用域 | 窗口增强模块 | 说明 |
| ------------ | --------------------------- | -------------------------------------------- |
| Alt+Tab 界面 | Q E | 左右切换多桌面 |
| Alt+Tab 界面 | W A S D | 上下左右切换窗口选择 |
| Alt+Tab 界面 | X C | 关闭选择的窗口（目前 X 和 C 没有区别） |
| Win+Tab 视图 | Alt + W A S D | 切换窗口选择 |
| 全局 | CapsLockX + Backspace | 删除当前桌面（会把所有窗口移到上一个桌面） |
| 全局 | CapsLockX + 1 2 ... 9 0 | 切换到第 n 个桌面 |
| 全局 | CapsLockX + Alt + 1 2 ... 9 0 | 把当前窗口移到第 n 个桌面 |
| 全局 | CapsLockX + Alt + = | 新建桌面，并把当前窗口移过去 |
| 全局 | CapsLockX + Alt + M | 快速堆叠当前桌面的窗口 |
| 全局 | CapsLockX + Alt + Shift + M | 快速堆叠当前桌面的窗口（包括最小化的窗口） |
| 全局 | CapsLockX + Alt + [ ] | 把当前窗口移到上一个/下一个桌面 |
| 全局 | CapsLockX + M | 快速排列当前桌面的窗口 |
| 全局 | CapsLockX + Shift + M | 快速排列当前桌面的窗口（包括最小化的窗口） |
| 全局 | CapsLockX + [ ] | 切换到上一个/下一个桌面 |
)")
; setup done

; flags
global ARRANGE_SIDE_BY_SIDE := 0
global ARRANGE_STACKED := 1 ; if not then arrange SIDE_BY_SIDE
global ARRANGE_MAXWINDOW := 2
global ARRANGE_MINWINDOW := 4
global ARRANGE_DEBUG := 8

Return

; 把当前窗口置顶

; #If !!(CapsLockXMode & CM_FN)

; 确保WinTab模块优先级比Mouse高，否则此处 WASD 无效
; Make sure WinTab module has higher priority than Mouse, otherwise WASD is invalid here

#if CapsLockXMode

; 自动排列窗口
c:: ArrangeWindows(ARRANGE_SIDE_BY_SIDE|ARRANGE_MAXWINDOW)
; 自动排列窗口（包括最小化的窗口）
^c:: ArrangeWindows(ARRANGE_SIDE_BY_SIDE|ARRANGE_MAXWINDOW|ARRANGE_MINWINDOW)
; 自动堆叠窗口
+c:: ArrangeWindows(ARRANGE_MAXWINDOW|ARRANGE_STACKED)
; 自动堆叠窗口（包括最小化的窗口）
^+c:: ArrangeWindows(ARRANGE_MAXWINDOW|ARRANGE_STACKED|ARRANGE_MINWINDOW)

; 使用 Windows 原生的方式自动排列窗口（和虚拟桌面不兼容）
; !o::
;     WinActivate ahk_class Shell_TrayWnd ahk_exe explorer.exe
;     ; side by side 排列
;     Send {AppsKey}i
Return

; WinTab 窗口切换
; \::
;     Send {LAlt Down}{Tab Down}{Tab Up}
;     KeyWait, \
;     Return
; \ Up::
;     Send {LAlt Down}{Tab Down}{Tab Up}
;     KeyWait, \
;     Return

; 切换当前窗口置顶并透明
+v::
    WinSet, Transparent, 200, A
    WinSet, Alwaysontop, Toggle, A
Return
; 让当前窗口临时透明
v::
    WinSet, Transparent, 100, A
    WinSet, Alwaysontop, On, A
Return
v Up::
    WinSet, Transparent, 255, A
    WinSet, Alwaysontop, Off, A
Return

; 关闭标签
x:: Send ^w
; 关闭窗口并切到下一窗口
+x::
    hWnd := WinActive("A")
    Send !{Esc}
    WM_CLOSE := 0x0010
    SendMessage, %WM_CLOSE%, 0, 0, , ahk_id %hWnd%
    ; ArrangeWindows(ARRANGE_SIDE_BY_SIDE|ARRANGE_MAXWINDOW)
Return
; 关闭窗口并切到下一窗口，并自动排列窗口
+!x::
    hWnd := WinActive("A")
    Send !{Esc}
    WM_CLOSE := 0x0010
    SendMessage, %WM_CLOSE%, 0, 0, , ahk_id %hWnd%
    ; ArrangeWindows(ARRANGE_MAXWINDOW|ARRANGE_STACKED)
Return
; 杀死窗口并切到下一窗口
^!x::
    hWnd := WinActive("A")
    Send !{Esc}
    WinKill ahk_id %hWnd%
Return

; Alt+Tab或 Win+Tab
#if MultitaskingViewFrameQ()

!1:: SwitchToDesktop(1)
!2:: SwitchToDesktop(2)
!3:: SwitchToDesktop(3)
!4:: SwitchToDesktop(4)
!5:: SwitchToDesktop(5)
!6:: SwitchToDesktop(6)
!7:: SwitchToDesktop(7)
!8:: SwitchToDesktop(8)
!9:: SwitchToDesktop(9)
!0:: SwitchToDesktop(10)
!-:: SwitchToDesktop(11)
!=:: SwitchToDesktop(12)

; 在 Win + Tab 下, WASD 模拟方向键, 1803之后还可以用
!a:: Left
!d:: Right
!w:: Up
!s:: Down

; cx 关闭应用
!c:: SendEvent {Blind}{Delete}{Right}
!x:: SendEvent {Blind}{Delete}{Right}

; 新建桌面
!z::
    SendEvent {Blind}{Esc}
    ; Sleep 200
    Send ^#d
Return

; 新建桌面并移动窗口
!v::
    SendEvent {Blind}{Esc}
    ; Sleep 200
    MoveActiveWindowWithAction("^#d")
Return

; 模拟 Tab 键切换焦点
; !\:: Send {Tab}
; 在 Win10 下的 Win+Tab 界面，WASD 切换窗口焦点
; 以及在窗口贴边后，WASD 切换窗口焦点

; 切换桌面概览
!q:: Send ^#{Left}
!e:: Send ^#{Right}
![:: Send ^#{Left}
!]:: Send ^#{Right}

; 增删桌面
=:: Send ^#d
-:: Send ^#{F4}
z:: Send ^#{F4}

#if False && "FUNCTIION DEFINES"

MultitaskingViewFrameQ(){
Return WinActive("ahk_class MultitaskingViewFrame") || WinActive("ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe")
}
; this is improved method for stable
GetMonitorIndexFromWindowByWindowsCenterPoint(hWnd){
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    CX := X + W / 2
    CY := Y + H / 2
    SysGet, monitorCount, MonitorCount
    monitorIndex := "" ; default
    loop %monitorCount% {
        SysGet, M, Monitor, %A_Index%
        ; Compare center position to determine the monitor index.
        if (( abs(min(max(MLeft, CX), MRight) - CX) <= 1)&& ( abs(min(max(MTop, CY), MBottom) - CY) <= 1)){
            msgbox, , %title%, %A_Index% %MLeft% %CX% %MRight% %EQ%
            monitorIndex := A_Index
            break
        }
    }
Return %monitorIndex%
}
; below function is modified from [How to determine a window is in which monitor? - Ask for Help - AutoHotkey Community]( https://autohotkey.com/board/topic/69464-how-to-determine-a-window-is-in-which-monitor/ )
GetMonitorIndexFromWindow(hWnd){
    ; default is 0 to prevent ...
    monitorIndex := ""
    VarSetCapacity(monitorInfo, 40)
    NumPut(40, monitorInfo)
    monitorHandle := DllCall("MonitorFromWindow", "uint", hWnd, "uint", 0x2)
    if (monitorHandle && DllCall("GetMonitorInfo", "uint", monitorHandle, "uint", &monitorInfo)){
        monitorLeft := NumGet(monitorInfo, 4, "Int")
        monitorTop := NumGet(monitorInfo, 8, "Int")
        monitorRight := NumGet(monitorInfo, 12, "Int")
        monitorBottom := NumGet(monitorInfo, 16, "Int")
        workLeft := NumGet(monitorInfo, 20, "Int")
        workTop := NumGet(monitorInfo, 24, "Int")
        workRight := NumGet(monitorInfo, 28, "Int")
        workBottom := NumGet(monitorInfo, 32, "Int")
        isPrimary := NumGet(monitorInfo, 36, "Int") & 1

        ; msgbox, , , workLeft%workLeft% workTop%workTop% workRight%workRight% workBottom%workBottom%

        SysGet, monitorCount, MonitorCount
        loop %monitorCount%
        {
            SysGet, tempMon, Monitor, %A_Index%
            ; Compare location to determine the monitor index.
            if ((monitorLeft = tempMonLeft) and (monitorTop = tempMonTop)and (monitorRight = tempMonRight) and (monitorBottom = tempMonBottom)){
                monitorIndex := A_Index
                break
            }
        }
    }
    if (monitorIndex){
        Return %monitorIndex%
    }
    monitorIndex := GetMonitorIndexFromWindowByWindowsCenterPoint(hWnd)
    if (monitorIndex){
        Return %monitorIndex%
    }
Return 1
}

ArrangeWindows(arrangeFlags = "0"){
    arrangeFlags += 0 ; string to number
    ; 常量定义
    WS_EX_TOOLWINDOW := 0x00000080
    WS_EX_APPWINDOW := 0x00040000
    WS_CAPTION := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE := 0x08000000
    WS_POPUP := 0x80000000

    SysGet, MonitorCount, MonitorCount
    loop %MonitorCount% {
        MonitorIndex := A_Index
        listOfWindow%MonitorIndex% := ""
    }

    DetectHiddenWindows, Off
    WinGet, id, List, , , 
    loop %id% {
        hWnd := id%A_Index%
        WinGet, this_pid, PID, ahk_id %hWnd%
        WinGet, style, style, ahk_id %hWnd%
        WinGetTitle, this_title, ahk_id %hWnd%
        WinGetClass, this_class, ahk_id %hWnd%
        ; Process, , PID-or-Name [, Param3]
        if (1){
            ; 黑名单
            ; ; 跳过无标题窗口
            ; if !(style & WS_CAPTION)
            ;     Continue
            ; ; 跳过工具窗口
            ; if (style & WS_EX_TOOLWINDOW)
            ;     Continue
            ; 只显示Alt+TAB里有的窗口
            if (!(style & WS_EX_APPWINDOW)){
                continue ; ; 跳过弹出窗口
            }
            ; if (style & WS_POPUP)
            ;     Continue
            ; 排除空标题窗口
            if (!RegExMatch(this_title, ".+")){
                Continue ; If (this_class == "Progman") ; Continue ; 排除 Win10 的常驻窗口管理器
            }
            ; 排除不归属于当前参数显示器的窗口
            ; if (!!MonitorIndex){
            ; this_monitor := GetMonitorIndexFromWindow(hWnd)
            ;     if (MonitorIndex != this_monitor){
            ;         continue
            ;     }
            ; }
            ; 跳过不在当前虚拟桌面的窗口
            if (!IsWindowOnCurrentVirtualDesktop(hWnd)){
                continue
            }
            ; 跳过最大化窗口
            WinGet, minmax, minmax, ahk_id %hWnd%
            if (minmax == 1 && !(arrangeFlags & ARRANGE_MAXWINDOW)){
                continue
            }
            ; 跳过最小化的窗口
            if (minmax == -1 && !(arrangeFlags & ARRANGE_MINWINDOW)){
                continue
            }
            ; 尝试跳过隐藏窗口
            GWL_STYLE := -16
            GWL_EXSTYLE := -20
            WS_STYLE := DllCall("GetWindowLong" (A_PtrSize=8 ? "Ptr" : ""), "Ptr", hWnd, "Int", GWL_STYLE, "PTR")
            WS_VISIBLE := 0x10000000
            if (!(style & WS_VISIBLE)){
                continue
            }
            ; 尝试跳过隐藏窗口
            if ( !DllCall("IsWindowVisible", "Ptr", hWnd, "PTR") ){
                continue
            }
            ; 跳过不可见的 UWP 窗口 
            WinGetClass, this_class, ahk_id %hWnd%
            if ( this_class == "ApplicationFrameWindow"){
                Continue
            }
            ; BOOL IsWindowVisible(HWND hWnd);
            if (0){
                ; debug
                ; WinHide, ahk_id %hWnd%
                WinGet, style_before, style, ahk_id %hWnd%
                ; WinShow, ahk_id %hWnd%
                ; WinActivate, ahk_id %hWnd%
                WinGet, style, style, ahk_id %hWnd%
                WinGet, style, style, ahk_id %hWnd%
                visible := !!(style & WS_VISIBLE)
                ; if (!visible){
                ;     WinShow, ahk_id %hWnd%c
                ; }
                WinGet, this_pid, PID, ahk_id %hWnd%
                WinGet, minmax, minmax, ahk_id %hWnd%
                WinGetClass, this_class, ahk_id %hWnd%
                WinGetTitle, this_title, ahk_id %hWnd%
                WinGetPos, X, Y, Width, Height, ahk_id %hWnd%
                ; DllCall("IsWindowVisible", "Ptr", hWnd, "PTR")
                msg := ""
                msg = %msg% arrangeFlags%arrangeFlags%`n
                msg = %msg% %A_Index% of %id%`n
                msg = %msg% ahk_id %hWnd%`n
                msg = %msg% ahk_class %this_class%`n
                msg = %msg% ahk_pid %this_pid%`n
                msg = %msg% %X% %Y% %Width% %Height%`n
                msg = %msg% title %this_title%`n
                msg = %msg% minmax %minmax%`n
                msg = %msg% style %style%`n
                msg = %msg% style_before %style_before%`n
                msg = %msg% visible %visible%`n
                msg = %msg% `nContinue?
                MsgBox, 4, , %msg%
                IfMsgBox, NO, break
                ; WinShow, ahk_id %hWnd%
                ; WinActivate, ahk_id %hWnd%
            }

        }
        this_monitor := GetMonitorIndexFromWindow(hWnd)
        listOfWindow%this_monitor% .= "ahk_pid " this_pid " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
        ; listOfWindow%this_monitor% .= "ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
        ; TrayTip, listOfWindow%this_monitor%, % listOfWindow%this_monitor%
    }
    ; TrayTip, DEBUG_AW MonitorCount, %MonitorCount%
    loop %MonitorCount% {
        ; 先按 pid 和 hwnd 排列，这样排出来的窗口的顺序就是稳定的了
        ; MsgBox, , , low %listOfWindow1%
        tooltip % listOfWindow
        Sort listOfWindow%A_Index%
        if (arrangeFlags & ARRANGE_STACKED){
            ArrangeWindowsStacked(listOfWindow%A_Index%, arrangeFlags, A_Index)
        } else {
            ArrangeWindowsSideBySide(listOfWindow%A_Index%, arrangeFlags, A_Index)
        }
    }
}
ArrangeWindowsSideBySide(listOfWindow, arrangeFlags = "0", MonitorIndex = ""){
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; TrayTip DEBUG_AW_listOfWindow_%n%, %listOfWindow%
    ; try parse work rect from monitor
    if (!MonitorIndex){
        AreaX := 0
        AreaY := 0
        AreaW := A_ScreenWidth
        AreaH := A_ScreenHeight
    } else {
        SysGet, MonitorWorkArea, MonitorWorkArea, %MonitorIndex%
        ; SysGet, Monitor, Monitor, %MonitorIndex%
        AreaX := MonitorWorkAreaLeft
        AreaY := MonitorWorkAreaTop
        AreaW := MonitorWorkAreaRight - MonitorWorkAreaLeft
        AreaH := MonitorWorkAreaBottom - MonitorWorkAreaTop
    }
    ; AreaH /= 2
    ; TrayTip DEBUG Area, %AreaX% %AreaY% %AreaW% %AreaH%
    ; calc rows and cols
    ; shorten edge first
    if (AreaW <= AreaH){
        ; row more than cols
        col := Sqrt(n) | 0
        row := Ceil(n / col)
    } else {
        ; col more than rows
        row := Sqrt(n) | 0
        col := Ceil(n / row)
    }
    size_x := AreaW / col
    size_y := AreaH / row
    k := n - 1
    lasthWnd := 0
    loop Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")

        ; 同一进程窗口长边优先排列
        if (AreaW >= AreaH){
            ; row first
            nx := Mod(k, col)
            ny := k / col | 0
        } else {
            ; col first
            nx := k / row | 0
            ny := Mod(k, row)
        }
        x := AreaX + nx * size_x
        y := AreaY + ny * size_y

        ; 填满窗口间的缝隙
        x:= x-8, y:=y, w:=size_x+16, h:=size_y+8

        ; 左上角不要出界，否则不同DPI的显示器连接处宽度计算不正常
        dX := max(AreaX - x, 0), x += dX, w -= dX
        dY := max(AreaY - y, 0), y += dY, h -= dY
        ; 右下角也不要出界，下边留出1px让wallpaper engine 的bgm放出来
        w := min(x + w, AreaX + AreaW) - x
        h := min(y + h, AreaY + AreaH - 1) - y

        FastResizeWindow(hWnd, x, y, w, h)
        lasthWnd := hWnd
        k-=1
    }
    WinGet, hWnd, , A
    ; DllCall( "FlashWindow", UInt, hWnd, Int, True )

    ; loop Parse, listOfWindow, `n
    ; {
    ;     hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
    ;     WinActivate ahk_id %hWnd%
    ; }
    Sleep, 1000
    SWP_NOACTIVATE := 0x0010
    SWP_ASYNCWINDOWPOS:= 0x4000
    SWP_NOMOVE := 0x0002
    SWP_NOSIZE := 0x0001
    lasthWnd := -2
    loop, Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        ; WinActivate ahk_id %hWnd%
        DllCall("SetWindowPos"
        , "UInt", hWnd ; handle
        , "UInt", lasthWnd ; z-index
        , "Int", 0 ;  x
        , "Int", 0 ; y
        , "Int", 0 ; width
        , "Int", 0 ; height
        , "UInt", SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOMOVE) ; SWP_ASYNCWINDOWPOS
        lasthWnd := hWnd
    }

}

ArrangeWindowsStacked(listOfWindow, arrangeFlags = "0", MonitorIndex = ""){
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; try parse work rect from monitor
    if (!MonitorIndex){
        AreaX := 0
        AreaY := 0
        AreaW := A_ScreenWidth
        AreaH := A_ScreenHeight
    } else {
        SysGet, MonitorWorkArea, MonitorWorkArea, %MonitorIndex%
        AreaX := MonitorWorkAreaLeft
        AreaY := MonitorWorkAreaTop
        AreaW := MonitorWorkAreaRight - MonitorWorkAreaLeft
        AreaH := MonitorWorkAreaBottom - MonitorWorkAreaTop
    }

    k := 0
    dx := 64
    dy := 64
    w := AreaW - 2 * dx - n * dx + dx
    h := AreaH - 2 * dy - n * dy + dy
    lasthWnd := -2
    loop, Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        ; fix hidden UWP ApplicationFrameWindow Window
        WinGetClass, this_class, ahk_id %hWnd%
        if (this_class == "ApplicationFrameWindow"){
            WinActivate, ahk_id %hWnd%
        }

        x := AreaX + (n - k) * dx
        y := AreaY + (n - k) * dy
        FastResizeWindow(hWnd, x, y, w, h)
        lasthWnd := hWnd
        ; FastResizeWindow(hWnd, x, y, w, h)
        k+=1
    }
    WinActivate ahk_id %lasthWnd%
    SWP_NOACTIVATE := 0x0010
    SWP_ASYNCWINDOWPOS:= 0x4000
    SWP_NOMOVE := 0x0002
    SWP_NOSIZE := 0x0001
    lasthWnd := -2
    loop, Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        ; WinActivate ahk_id %hWnd%
        DllCall("SetWindowPos"
        , "UInt", hWnd ; handle
        , "UInt", lasthWnd ; z-index
        , "Int", 0 ;  x
        , "Int", 0 ; y
        , "Int", 0 ; width
        , "Int", 0 ; height
        , "UInt", SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOMOVE | SWP_ASYNCWINDOWPOS) ; SWP_ASYNCWINDOWPOS
        lasthWnd := hWnd
    }

    ; loop, Parse, listOfWindow, `n
    ; {
    ;     hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
    ;     WinActivate ahk_id %hWnd%
    ; }
}
FastResizeWindow(hWnd, x, y, w, h, Active := 0, zIndex := 0){
    ; 如有必要则还原最大化最小化的窗口
    WinGet, minmax, minmax, ahk_id %hWnd%
    if (minmax != 0){
        WinRestore, ahk_id %hWnd%
        ; needSetTOPMOST := 1
    }
    ; ref: [SetWindowPos function (winuser.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos )
    HWND_TOPMOST := -1
    HWND_BOTTOM := 1
    HWND_TOP := 0
    HWND_NOTOPMOST := -2
    SWP_NOACTIVATE := 0x0010
    SWP_ASYNCWINDOWPOS:= 0x4000
    SWP_NOZORDER := 0x0004
    SWP_NOMOVE := 0x0002
    SWP_NOSIZE := 0x0001
    ; 先置顶（否则会显示在最大化窗口的后面 -- 被挡住）
    if (Active){
        WinActivate ahk_id %hWnd%
    }
    if (zIndex){
        DllCall("SetWindowPos"
        , "UInt", hWnd ; handle
        , "UInt", zIndex ; z-index
        , "Int", x ;  x
        , "Int", y ; y
        , "Int", w ; width
        , "Int", h ; height
        , "UInt", SWP_NOACTIVATE) ; SWP_ASYNCWINDOWPOS
    } else {
        DllCall("SetWindowPos"
        , "UInt", hWnd ;handle
        , "UInt", 0 ; z-index
        , "Int", x
        , "Int", y
        , "Int", w
        , "Int", h
        , "UInt", SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS) ; SWP_ASYNCWINDOWPOS
    }
}
