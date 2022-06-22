; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：窗口增强模块
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.07.04
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; 许可证 LICENCE: GNU GPLv3 ( https://www.gnu.org/licenses/gpl-3.0.html )
; ========== CapsLockX ==========
;
; Exit if running without CapsLockX
;

if (!CapsLockX) {
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

; setup done
; flags
global 上次CtrlShiftAlt时刻 := 0
global 上次CtrlShiftAlt锁 := 0
global FLAG_CtrlShiftAlt按下 := 0j

global ARRANGE_SIDE_BY_SIDE := 0x00
; if not then arrange SIDE_BY_SIDE
global ARRANGE_STACKED := 0x01
global ARRANGE_MAXWINDOW := 0x02
global ARRANGE_MINWINDOW := 0x04
global ARRANGE_DEBUG := 0x08
global ARRANGE_MOVING := 0x10
global ARRANGE_Z_ORDERING := 0x20

global lastFlashWinIDs := []
global 最迟闪动窗口 := {}
global 窗口鼠标位置表表 := {}
global T窗口增强_鼠标位置记忆 := CapsLockX_Config("窗口增强", "鼠标位置记忆尝试", 1, "在CLX+Z窗口切换时记住还原鼠标在每个窗口中的位置")

闪动窗口记录器初始化()

Return

闪动窗口记录器初始化()
{
    Gui +LastFound
    hWnd := WinExist(), DllCall( "RegisterShellHookWindow", UInt, hWnd )
    MsgNum := DllCall( "RegisterWindowMessage", Str, "SHELLHOOK" )
    OnMessage( MsgNum, "ShellMessage" )
}

#if False && "FUNCTIION DEFINES"

AltTabWindowGet()
{
    return WinActive("ahk_class MultitaskingViewFrame") || WinActive("ahk_class XamlExplorerHostIslandWindow")
}
WinTabWindowGet()
{
    return WinActive("ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe") || WinActive("ahk_class XamlExplorerHostIslandWindow")
}
多任务窗口切换界面内()
{
    Return AltTabWindowGet() || WinTabWindowGet()
}
; this is improved method for stable
GetMonitorIndexFromWindowByWindowsCenterPoint(hWnd)
{
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    CX := X + W / 2
    CY := Y + H / 2
    SysGet, monitorCount, MonitorCount
    MonitorIndex := "" ; default
    loop %monitorCount% {
        SysGet, M, Monitor, %A_Index%
        ; Compare center position to determine the monitor index.
        if (( abs(min(max(MLeft, CX), MRight) - CX) <= 1)&& ( abs(min(max(MTop, CY), MBottom) - CY) <= 1)) {
            msgbox, , %title%, %A_Index% %MLeft% %CX% %MRight% %EQ%
            MonitorIndex := A_Index
            break
        }
    }
    Return %MonitorIndex%
}
; below function is modified from [How to determine a window is in which monitor? - Ask for Help - AutoHotkey Community]( https://autohotkey.com/board/topic/69464-how-to-determine-a-window-is-in-which-monitor/ )
GetMonitorIndexFromWindow(hWnd)
{
    ; default is 0 to prevent ...
    MonitorIndex := ""
    VarSetCapacity(monitorInfo, 40)
    NumPut(40, monitorInfo)
    monitorHandle := DllCall("MonitorFromWindow", "uint", hWnd, "uint", 0x2)
    if (monitorHandle && DllCall("GetMonitorInfo", "uint", monitorHandle, "uint", &monitorInfo)) {
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
            if ((monitorLeft = tempMonLeft) and (monitorTop = tempMonTop)and (monitorRight = tempMonRight) and (monitorBottom = tempMonBottom)) {
                MonitorIndex := A_Index
                break
            }
        }
    }
    if (MonitorIndex) {
        Return %MonitorIndex%
    }
    MonitorIndex := GetMonitorIndexFromWindowByWindowsCenterPoint(hWnd)
    if (MonitorIndex) {
        Return %MonitorIndex%
    }
    Return 1
}
WindowsListOfMonitorFast(arrangeFlags, MonitorIndex := 0)
{
    windowsMatches := ""
    ; 常量定义
    WS_EX_TOOLWINDOW := 0x00000080
    WS_EX_APPWINDOW := 0x00040000
    WS_CAPTION := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE := 0x08000000
    WS_POPUP := 0x80000000
    WS_VISIBLE := 0x10000000
    
    DetectHiddenWindows, Off
    WinGet, id, List, , , 
    loop %id% {
        hWnd := id%A_Index%
        WinGet, style, style, ahk_id %hWnd%
        ; 跳过无标题窗口
        if (!(style & WS_CAPTION)) {
            Continue
        }
        ; 跳过工具窗口
        if (style & WS_EX_TOOLWINDOW) {
            Continue
        }
        if (style & WS_POPUP) {
            Continue
        }
        ; 只显示Alt+TAB里有的窗口
        if (!(style & WS_EX_APPWINDOW)) {
            continue ; ; 跳 过弹出窗口
        }
        ; 尝试跳过隐藏窗口
        if (!(style & WS_VISIBLE)) {
            continue
        }
        ; 跳过不在当前虚拟桌面的窗口
        if (!IsWindowOnCurrentVirtualDesktop(hWnd)) {
            continue
        }
        ; 排除不归属于当前参数显示器的窗口
        if (!!MonitorIndex) {
            this_monitor := GetMonitorIndexFromWindow(hWnd)
            if (MonitorIndex != this_monitor) {
                continue
            }
        }
        WinGet, this_exe, ProcessName, ahk_id %hWnd%
        windowsMatches .= "ahk_exe " this_exe " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
        ; windowsMatches .= "ahk_pid " this_pid " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
    }
    return windowsMatches
}
WindowsListOfMonitorInCurrentDesktop(arrangeFlags, MonitorIndex := 0)
{
    windowsMatches := ""
    ; 常量定义
    WS_EX_TOOLWINDOW := 0x00000080
    WS_EX_APPWINDOW := 0x00040000
    WS_CAPTION := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE := 0x08000000
    WS_POPUP := 0x80000000
    
    DetectHiddenWindows, Off
    WinGet, id, List, , , 
    loop %id% {
        hWnd := id%A_Index%
        WinGet, style, style, ahk_id %hWnd%
        ; ; 跳过无标题窗口
        ; if !(style & WS_CAPTION)
        ;     Continue
        ; ; 跳过工具窗口
        ; if (style & WS_EX_TOOLWINDOW)
        ;     Continue
        ; if (style & WS_POPUP)
        ;     Continue
        ; 只显示Alt+TAB里有的窗口
        if (!(style & WS_EX_APPWINDOW)) {
            continue ; ; 跳 过弹出窗口
        }
        ; 尝试跳过隐藏窗口
        GWL_STYLE := -16
        GWL_EXSTYLE := -20
        ; WS_STYLE := DllCall("GetWindowLong" (A_PtrSize=8 ? "Ptr" : ""), "Ptr", hWnd, "Int", GWL_STYLE, "PTR")
        WS_VISIBLE := 0x10000000
        if (!(style & WS_VISIBLE)) {
            continue
        }
        ; 跳过不在当前虚拟桌面的窗口
        if (!IsWindowOnCurrentVirtualDesktop(hWnd)) {
            continue
        }
        ; 排除不归属于当前参数显示器的窗口
        if (!!MonitorIndex) {
            this_monitor := GetMonitorIndexFromWindow(hWnd)
            if (MonitorIndex != this_monitor) {
                continue
            }
        }
        ; 尝试跳过隐藏窗口
        if ( !DllCall("IsWindowVisible", "Ptr", hWnd, "PTR") ) {
            continue
        }
        ; 跳过最大化窗口
        WinGet, minmax, minmax, ahk_id %hWnd%
        if (minmax == 1 && !(arrangeFlags & ARRANGE_MAXWINDOW)) {
            continue
        }
        ; 跳过最小化的窗口
        if (minmax == -1 && !(arrangeFlags & ARRANGE_MINWINDOW)) {
            continue
        }
        WinGetTitle, this_title, ahk_id %hWnd%
        ; 排除空标题窗口
        if (!RegExMatch(this_title, ".+")) {
            Continue ; If (this_class == "Progman") ; Continue ; 排除 Win10 的常驻窗口管理器
        }
        ; 跳过不可见的 UWP 窗口
        WinGetClass, this_class, ahk_id %hWnd%
        if ( this_class == "ApplicationFrameWindow") {
            Continue
        }
        
        WinGet, this_exe, ProcessName, ahk_id %hWnd%
        windowsMatches .= "ahk_exe " this_exe " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
        ; windowsMatches .= "ahk_pid " this_pid " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
    }
    Sort windowsMatches, R
    return windowsMatches
}
WindowsWalkToDirection右上左下(arrangeFlags = "0", direction := 0)
{
    ; 列出所有窗口
    static listOfWindow_cache := ""
    static listOfWindow_cache_time := 0
    if (listOfWindow_cache_time + 5000 < A_TickCount ) {
        listOfWindow_cache := ""
    }
    if (listOfWindow_cache ) {
        listOfWindow := listOfWindow_cache
    } else {
        listOfWindow := WindowsListOfMonitorFast(arrangeFlags) ; 目前这个函数扔然是最大的性能瓶颈
        listOfWindow_cache := listOfWindow
        listOfWindow_cache_time := A_TickCount
    }
    ; tooltip %listOfWindow%
    
    ; 相对当前窗口的位置计算
    hWnd := WinActive("A")
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    this_CX := X + W / 2
    this_CY := Y + H / 2
    最优距离 := 0
    最优方向 := ""k
    最优HWND := hWnd
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    loop Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        if (!hWnd) {
            continue
        }
        WinGetPos, X, Y, W, H, ahk_id %hWnd%
        CX := X + W / 2
        CY := Y + H / 2
        DX := CX - this_CX
        DY := CY - this_CY
        cos45 := Cos(-45 / 180 * 3.1415926535)
        sin45 := Sin(-45 / 180 * 3.1415926535)
        rotatedDX := DX * cos45 - DY * sin45
        rotatedDY := -(DX * sin45 + DY * cos45)
        方向 := ""
        if (rotatedDX > 0 && rotatedDY > 0) {
            方向 := "右"
        }
        if (rotatedDX < 0 && rotatedDY > 0) {
            方向 := "上"
        }
        if (rotatedDX < 0 && rotatedDY < 0) {
            方向 := "左"
        }
        if (rotatedDX > 0 && rotatedDY < 0) {
            方向 := "下"
        }
        距离 := ( DX**2 + DY**2 ) ** (1/2)
        ; WinGetTitle, 当前标题, ahk_id %hWnd%
        ; msgbox %当前标题% `n 方向 %方向% %距离% `n %DX% %DY% `n %rotatedDX% %rotatedDY%
        if (距离 && (距离 < 最优距离 || !最优距离) && ( 0
        || (direction == 1 && 方向 == "右")
        || (direction == 2 && 方向 == "上")
        || (direction == 3 && 方向 == "左")
        || (direction == 4 && 方向 == "下")))
        {
            最优HWND := hWnd
            最优距离 := 距离
            最优方向 := 方向
        }
    }
    if (最优距离) {
        WinGetTitle, Title, ahk_id %最优HWND%
        WinActivate, ahk_id %最优HWND%
        ; TrayTip, CapsLockX 窗口增强, 切换到窗口 %Title% `n 方向 %最优方向% `n 距离 %最优距离%
        return True
    }
    return False
}
ArrangeWindows(arrangeFlags = "0")
{
    arrangeFlags += 0 ; string to number
    SysGet, MonitorCount, MonitorCount
    ; 列出每个显示器内的窗口
    loop %MonitorCount% {
        MonitorIndex := A_Index
        listOfWindow_%MonitorIndex% := WindowsListOfMonitorInCurrentDesktop(arrangeFlags, MonitorIndex)
    }
    ; 位置调整
    loop %MonitorCount% {
        MonitorIndex := A_Index
        if (arrangeFlags & ARRANGE_STACKED) {
            ArrangeWindowsStacked(listOfWindow_%MonitorIndex%, arrangeFlags | ARRANGE_MOVING, MonitorIndex)
        } else {
            ArrangeWindowsSideBySide(listOfWindow_%MonitorIndex%, arrangeFlags | ARRANGE_MOVING, MonitorIndex)
        }
    }
    ; Z_Order 调整
    loop %MonitorCount% {
        MonitorIndex := A_Index
        if (arrangeFlags & ARRANGE_STACKED) {
            ArrangeWindowsStacked(listOfWindow_%MonitorIndex%, arrangeFlags | ARRANGE_Z_ORDERING, MonitorIndex)
        } else {
            ArrangeWindowsSideBySide(listOfWindow_%MonitorIndex%, arrangeFlags | ARRANGE_Z_ORDERING, MonitorIndex)
        }
    }
}
ArrangeWindowsSideBySide(listOfWindow, arrangeFlags = "0", MonitorIndex = "")
{
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; TrayTip DEBUG_AW_listOfWindow_%n%, %listOfWindow%
    ; try parse work rect from monitor
    if (!MonitorIndex) {
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
    if (arrangeFlags & ARRANGE_MOVING) {
        ; AreaH /= 2
        ; TrayTip DEBUG Area, %AreaX% %AreaY% %AreaW% %AreaH%
        ; calc rows and cols
        ; shorten edge first
        if (AreaW <= AreaH) {
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
            if (AreaW >= AreaH) {
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
        ; Sleep, 1000
    }
    if (arrangeFlags & ARRANGE_Z_ORDERING) {
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
}
ArrangeWindowsStacked(listOfWindow, arrangeFlags = "0", MonitorIndex = "")
{
    
    dx := 96
    dy := 96
    
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; try parse work rect from monitor
    if (!MonitorIndex) {
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
    
    if (arrangeFlags & ARRANGE_MOVING) {
        k := 0
        w := AreaW - 2 * dx - n * dx + dx
        h := AreaH - 2 * dy - n * dy + dy
        lasthWnd := -2
        loop, Parse, listOfWindow, `n
        {
            hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
            ; fix hidden UWP ApplicationFrameWindow Window
            WinGetClass, this_class, ahk_id %hWnd%
            if (this_class == "ApplicationFrameWindow") {
                WinActivate, ahk_id %hWnd%
            }
            
            x := AreaX + (n - k) * dx
            y := AreaY + (n - k) * dy
            FastResizeWindow(hWnd, x, y, w, h)
            lasthWnd := hWnd
            ; FastResizeWindow(hWnd, x, y, w, h)
            k+=1
        }
    }
    if (arrangeFlags & ARRANGE_Z_ORDERING) {
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
            , "Int", 0 ; x
            , "Int", 0 ; y
            , "Int", 0 ; width
            , "Int", 0 ; height
            , "UInt", SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOMOVE | SWP_ASYNCWINDOWPOS) ; SWP_ASYNCWINDOWPOS
            lasthWnd := hWnd
        }
        
    }
    ; loop, Parse, listOfWindow, `n
    ; {
    ;     hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
    ;     WinActivate ahk_id %hWnd%
    ; }
}
FastResizeWindow(hWnd, x, y, w, h, Active := 0, zIndex := 0)
{
    ; 如有必要则还原最大化最小化的窗口
    WinGet, minmax, minmax, ahk_id %hWnd%
    if (minmax != 0) {
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
    if (Active) {
        WinActivate ahk_id %hWnd%
    }
    if (zIndex) {
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
CurrentWindowSetAsBackground()
{
    ; 后置当前窗口
    WinGet hWnd, id, A
    上次mstsc窗口hWnd := hWnd
    WinSet Bottom, , ahk_id %hWnd%
    ; 激活任务栏，夺走焦点
    WinActivate ahk_class Shell_TrayWnd
}

; 把当前窗口置顶

; 确保WinTab模块优先级比Mouse高，否则此处 WASD 无效
; Make sure WinTab module has higher priority than Mouse, otherwise WASD is invalid here

#if CapsLockXMode
    
z:: 最近1分钟内闪动窗口激活()
; z:: 上一个窗口激活()
+z:: 下一个窗口激活()
; !z:: 最近1分钟内闪动窗口激活()
x:: Send ^w ; 关闭标签
+x:: 关闭窗口并切到下一窗口()
^!x:: 杀死窗口并切到下一窗口()
c:: ArrangeWindows(ARRANGE_SIDE_BY_SIDE|ARRANGE_MAXWINDOW) ; 自动排列窗口
^c:: ArrangeWindows(ARRANGE_SIDE_BY_SIDE|ARRANGE_MAXWINDOW|ARRANGE_MINWINDOW) ; 自动排列窗口（包括最小化的窗口）
+c:: ArrangeWindows(ARRANGE_MAXWINDOW|ARRANGE_STACKED) ; 自动堆叠窗口
^+c:: ArrangeWindows(ARRANGE_MAXWINDOW|ARRANGE_STACKED|ARRANGE_MINWINDOW) ; 自动堆叠窗口（包括最小化的窗口）
+v:: 当前窗口置顶透明切换()
v:: 当前窗口临时透明()
v Up:: 当前窗口临时透明取消()
b:: 任务栏任务切换()

当前窗口置顶透明切换()
{
    WinSet, Transparent, 200, A
    WinSet, Alwaysontop, Toggle, A
}
当前窗口临时透明()
{
    WinSet, Transparent, 100, A
    WinSet, Alwaysontop, On, A
}
当前窗口临时透明取消()
{
    WinSet, Transparent, 255, A
    WinSet, Alwaysontop, Off, A
}
关闭窗口并切到下一窗口()
{
    hWnd := WinActive("A")
    Send !{Esc}
    WM_CLOSE := 0x0010
    SendMessage %WM_CLOSE%, 0, 0, , ahk_id %hWnd%
}
杀死窗口并切到下一窗口()
{
    hWnd := WinActive("A")
    Send !{Esc}
    WinKill ahk_id %hWnd%
}

; 使普通的按方向窗口切换的热键与CapsLockXMode互不干扰
#if !CapsLockXMode
    
#+h:: 窗口向右上左下方显示器移动(3)
#+l:: 窗口向右上左下方显示器移动(1)
#+j:: 窗口向右上左下方显示器移动(4)
#+k:: 窗口向右上左下方显示器移动(2)

窗口向右上左下方显示器移动(方向)
{
    WinWaitNotActive ahk_class MultitaskingViewFrame
    WinWaitNotActive ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe
    WindowsWalkToDirection右上左下(ARRANGE_MAXWINDOW, 方向)
}

; Alt+Tab 或 Win+Tab
#if 多任务窗口切换界面内()

; !1:: SwitchToDesktop(1)
; !2:: SwitchToDesktop(2)
; !3:: SwitchToDesktop(3)
; !4:: SwitchToDesktop(4)
; !5:: SwitchToDesktop(5)
; !6:: SwitchToDesktop(6)
; !7:: SwitchToDesktop(7)
; !8:: SwitchToDesktop(8)
; !9:: SwitchToDesktop(9)
; !0:: SwitchToDesktop(10)

; Move the current window to the X-th desktop
CLX_MoveCurrentWindowTo(x)
{
    SendEvent {Space}
    WinWaitNotActive ahk_class MultitaskingViewFrame
    WinWaitNotActive ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe
    MoveActiveWindowToDesktop(x)
    SendEvent !{Tab}
}

!1:: CLX_MoveCurrentWindowTo(1) ; 选中窗口移动到1号桌面
!2:: CLX_MoveCurrentWindowTo(2) ; 选中窗口移动到2号桌面
!3:: CLX_MoveCurrentWindowTo(3) ; 选中窗口移动到3号桌面
!4:: CLX_MoveCurrentWindowTo(4) ; 选中窗口移动到4号桌面
!5:: CLX_MoveCurrentWindowTo(5) ; 选中窗口移动到5号桌面
!6:: CLX_MoveCurrentWindowTo(6) ; 选中窗口移动到6号桌面
!7:: CLX_MoveCurrentWindowTo(7) ; 选中窗口移动到7号桌面
!8:: CLX_MoveCurrentWindowTo(8) ; 选中窗口移动到8号桌面
!9:: CLX_MoveCurrentWindowTo(9) ; 选中窗口移动到9号桌面
!0:: CLX_MoveCurrentWindowTo(10) ; 选中窗口移动到10号桌面

; 在 Win + Tab 下, WASD 模拟方向键, 1803之后还可以用
!a:: Left        ; 左
!d:: Right       ; 右
!w:: Up          ; 上
!s:: Down        ; 下
!r:: Volume_Up   ; 音量+
!f:: Volume_Down ; 音量-

; cx 关闭应用
!c:: SendEvent {Blind}{Delete}{Right} ; 关闭应用
!x:: SendEvent {Blind}{Delete}{Right} ; 关闭应用

; 切换多桌面
!q:: Send ^#{Left} ; 向左切换多桌面
!e:: Send ^#{Right} ; 向右切换多桌面

#if
    CtrlShiftAlt按下() {
    FLAG_CtrlShiftAlt按下 := 1
}
~$<!<+LCtrl:: CtrlShiftAlt按下()
~$<^<+LAlt:: CtrlShiftAlt按下()
~$<!<^LShift:: CtrlShiftAlt按下()

~$<!<+LCtrl Up:: CtrlShiftAlt弹起()
~$<^<+LAlt Up:: CtrlShiftAlt弹起()
~$<!<^LShift Up:: CtrlShiftAlt弹起()
CtrlShiftAlt弹起() {
    if (!(A_PriorKey == "LCtrl" || A_PriorKey == "LAlt" || A_PriorKey == "LShift")) {
        Return
    } ; 防止重复触发
    if (上次CtrlShiftAlt锁) {
        return
    }
    if (!FLAG_CtrlShiftAlt按下) {
        return
    }
    FLAG_CtrlShiftAlt按下 := 0
    
    上次CtrlShiftAlt锁 := 1
    ToolTip, % "双击 LCtrl LAlt LShift 来最后置当前窗口（主要用于虚拟机和远程桌面）"
    SetTimer, 窗口增强_RemoveToolTip, -1024
    现在 := A_TickCount
    间隔 := 现在 - 上次CtrlShiftAlt时刻
    if (间隔 < 200) {
        CurrentWindowSetAsBackground()
    } else {
        上次CtrlShiftAlt时刻 := 现在
    }
    上次CtrlShiftAlt锁 := 0
    return
}
窗口增强_RemoveToolTip()
{
    ToolTip
}

#if 任务栏中()

任务栏中()
{
    return WinActive("ahk_class Shell_TrayWnd ahk_exe explorer.exe")
}

^w:: 任务栏中关闭窗口()
Delete:: 任务栏中关闭窗口()

#if CapsLockXMode && 任务栏中()

x:: 任务栏中关闭窗口()

#if
    
任务栏中关闭窗口()
{
    SendEvent {AppsKey}
    任务栏匹配串 := "ahk_class Shell_TrayWnd ahk_exe explorer.exe"
    WinWaitNotActive %任务栏匹配串%, , 3
    if (ErrorLevel)
        Return
    SendEvent {Up}
}

任务栏任务切换()
{
    任务栏匹配串 := "ahk_class Shell_TrayWnd ahk_exe explorer.exe"
    if (WinActive(任务栏匹配串)) {
        WinGetPos, X, Y, W, H, %任务栏匹配串%
        if (W > H) {
            ControlSend, MSTaskListWClass1, {Left}, %任务栏匹配串%
        } else {
            ControlSend, MSTaskListWClass1, {Up}, %任务栏匹配串%
        }
    } else {
        WinActivate, %任务栏匹配串%
        ControlFocus, MSTaskListWClass1, %任务栏匹配串%
        ControlSend, MSTaskListWClass1, {End}, %任务栏匹配串%
    }
}

ReverseArray(oArray)
{
    Array := Object()
    for i, v in oArray {
        Array[oArray.Length()-i+1] := v
    }
    Return Array
}

arrayDistinctKeepTheLastOne(arr)
{
    ; Hash O(n)
    hash := {}, newArr := []
    rarr := ReverseArray(arr)
    for e, v in rarr {
        if (!hash.Haskey(v)) {
            hash[(v)] := 1, newArr.push(v)
        }
    }
    return ReverseArray(newArr)
}
窗口增强_UnixTimeStamp() {
    Static UnixStart := 116444736000000000
    DllCall("GetSystemTimeAsFileTime", "Int64P", FileTime)
    Return ((FileTime - UnixStart) // 10000000) - 27 ; currently (2019-01-21) 27 leap seconds have been added
}
ShellMessage( wParam, lParam )
{
    HSHELL_FLASH := 0x8006 ;  0x8006 is 32774 as shown in Spy!
    if (wParam = HSHELL_FLASH) {
        global lastFlashWinIDs
        hWnd := lParam
        lastFlashWinIDs.Push(hWnd)
        lastFlashWinIDs := arrayDistinctKeepTheLastOne(lastFlashWinIDs)
        ; lastFlashWinIDs.__Set(hWnd)
        ; WinGetTitle, this_title, ahk_id %hWnd%
        ; TrayTip, blinking, %this_title% is blinking
        TimeStamp := 窗口增强_UnixTimeStamp()
        最迟闪动窗口 := {hWnd: hWnd, TimeStamp: TimeStamp}
    }
}
; activate
鼠标位置记忆尝试()
{
    if (!T窗口增强_鼠标位置记忆) {
        return
    }
    ; 相对窗口坐标记忆位置
    CoordMode, Mouse, Window
    MouseGetPos, X, Y, hWnd, hWndCtrl
    CoordMode, Mouse, Screen
    窗口鼠标位置表表[hWnd] := {X: X, Y: Y, hWnd: hWnd, hWndCtrl: hWndCtrl}
}
鼠标位置还原尝试(hWnd:=0)
{
    if (!T窗口增强_鼠标位置记忆) {
        return
    }
    if (!hWnd)
        WinGet, hWnd, id, A
    hWndRecorded := 窗口鼠标位置表表[hWnd].hWnd
    if (hWndRecorded) {
        X := 窗口鼠标位置表表[hWnd].X, Y := 窗口鼠标位置表表[hWnd].Y
        ; 相对窗口坐标还原鼠标
        WinActivate, ahk_id %hWnd%
        WinGetPos, wX, wY, wW, wH, ahk_id %hWnd%
        X := wX + X, Y := wY + Y
        CoordMode, Mouse, Screen
        MouseMove, %X%, %Y%, 0
    } else {
        WinActivate, ahk_id %hWnd%
        WinGetPos, wX, wY, wW, wH, ahk_id %hWnd%
        X := wX+wW/2, Y := wY+wH/2
        CoordMode, Mouse, Screen
        MouseMove, %X%, %Y%, 0
    }
}
最近1分钟内闪动窗口激活()
{
    TimeStampNow := 窗口增强_UnixTimeStamp()
    hWnd         := 最迟闪动窗口.hWnd
    TimeStamp    := 最迟闪动窗口.TimeStamp
    if (hWnd && TimeStampNow - TimeStamp <= 60) {
        鼠标位置记忆尝试()
        WinActivate, ahk_id %hWnd%
        WinGetTitle, this_title, ahk_id %hWnd%
        TrayTip, 最近1分钟内闪动窗口激活, %this_title%
        最迟闪动窗口 := {}
        鼠标位置还原尝试()
    } else {
        上一个窗口激活()
    }
}
下一个窗口激活()
{
    鼠标位置记忆尝试()
    WinGet, hWnd, id, A
    SendEvent !{Esc}
    WinWaitNotActive ahk_id %hWnd%, , 1
    if (ErrorLevel) {
        return
    }
    鼠标位置还原尝试()
}
上一个窗口激活()
{
    鼠标位置记忆尝试()
    WinGet, hWnd, id, A
    SendEvent +!{Esc}
    WinWaitNotActive ahk_id %hWnd%, , 1
    if (ErrorLevel) {
        return
    }
    鼠标位置还原尝试()
}
最迟闪动窗口激活()
{
    ; @deprecated
    鼠标位置记忆尝试()
    while % lastFlashWinIDs.Count()
    {
        hWnd := WinExist("ahk_id " lastFlashWinIDs.Pop())
        if (hWnd) {
            WinActivate, ahk_id %hWnd%
            WinGetTitle, this_title, ahk_id %hWnd%
            TrayTip, switched, switched to blinking %this_title%
            鼠标位置还原尝试(hWnd)
            Return
        }
    }
    SendEvent +!{Esc}
    WinWaitNotActive ahk_id %hWnd%, , 1
    if (ErrorLevel) {
        return
    }
    鼠标位置还原尝试()
}